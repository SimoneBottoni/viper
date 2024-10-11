use anyhow::anyhow;
use ark_bls12_381::{Bls12_381, Config, Fr, G1Projective, G2Projective};
use ark_ec::bls12::Bls12;
use ark_ec::pairing::{Pairing, PairingOutput};
use ark_ec::Group;
use ark_ff::Field;
use ark_serialize::CanonicalSerializeHashExt;
use ark_std::UniformRand;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::thread_rng;
use rayon::prelude::*;
use sha2::Sha256;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Lam {
    client_id: u64,
    sig: ed25519_dalek::Signature,
    big_z: G2Projective,
    big_a: G1Projective,
    big_c: G1Projective,
}

impl Lam {
    fn clone_empty(&self) -> Lam {
        Self {
            big_a: G1Projective::default(),
            big_c: G1Projective::default(),
            ..self.clone()
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Signature {
    lams: Vec<Lam>,
    big_r: G1Projective,
    big_s: G2Projective,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SK {
    k: u64,
    sk_sig: SigningKey,
    xs: Vec<Fr>,
    y: Fr,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PK {
    pk_sig: VerifyingKey,
    hs: Vec<PairingOutput<Bls12<Config>>>,
    big_y: G2Projective,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyPair {
    pub sk: SK,
    pub pk: PK,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Mkhs {
    g1: G1Projective,
    g2: G2Projective,
    gt: PairingOutput<Bls12<Config>>,
    n: usize,
    t: usize,
    big_hs: Vec<G1Projective>,
}

impl Mkhs {
    pub fn setup(n: usize, t: usize) -> Mkhs {
        let g1 = G1Projective::generator();
        let g2 = G2Projective::generator();
        let gt = Bls12_381::pairing(g1, g2);

        let big_hs = (0..t)
            .into_par_iter()
            .map(|_| G1Projective::rand(&mut thread_rng()))
            .collect();

        Mkhs {
            g1,
            g2,
            gt,
            n,
            t,
            big_hs,
        }
    }

    pub fn generate_keys(&self, client_id: u64) -> KeyPair {
        let sk_sig = SigningKey::generate(&mut thread_rng());
        let pk_sig = sk_sig.verifying_key();

        let xs = vec![Fr::rand(&mut thread_rng()); self.n];

        let y = Fr::rand(&mut thread_rng());
        let big_y = self.g2 * y;

        let hs = xs.par_iter().map(|v| self.gt * v).collect();

        KeyPair {
            sk: SK {
                k: client_id,
                sk_sig,
                xs,
                y,
            },
            pk: PK { pk_sig, hs, big_y },
        }
    }

    pub fn sign(&self, sk: &SK, message: &[Fr]) -> Signature {
        let z = Fr::from(sk.k);
        let big_z = self.g2 * z;

        let temp = big_z.hash::<Sha256>();
        let sig = sk.sk_sig.sign(&temp);

        let r = Fr::rand(&mut thread_rng());
        let s = Fr::rand(&mut thread_rng());

        let mut big_a = self.g1 * (sk.xs[0] + r);
        let mut big_c = self.g1 * s;

        for (i, v) in self.big_hs.iter().enumerate() {
            big_a += *v * (sk.y + message[i]);
            big_c += *v * message[i]
        }

        big_a *= z.inverse().unwrap();

        let big_r = self.g1 * (r - (sk.y + s));
        let big_s = self.g2 * (-s);

        Signature {
            lams: vec![Lam {
                client_id: sk.k,
                sig,
                big_z,
                big_a,
                big_c,
            }],
            big_r,
            big_s,
        }
    }

    pub fn eval(&self, sigs: &[Signature]) -> Signature {
        let mut big_r = G1Projective::default();
        let mut big_s = G2Projective::default();

        let mut id_lam = HashMap::new();
        for sig in sigs.iter().take(self.n) {
            big_r += sig.big_r;
            big_s += sig.big_s;

            for lam in sig.lams.iter() {
                let mut temp = lam.clone_empty();
                temp.big_a += lam.big_a;
                temp.big_c += lam.big_c;

                id_lam.insert(temp.client_id, temp);
            }
        }

        Signature {
            lams: id_lam.values().cloned().collect(),
            big_r,
            big_s,
        }
    }

    pub fn verify(
        &self,
        pks: &HashMap<u64, PK>,
        messages: &[Fr],
        signature: &Signature,
    ) -> anyhow::Result<()> {
        signature.lams.par_iter().try_for_each(|v| {
            let key = pks
                .get(&v.client_id)
                .ok_or_else(|| anyhow!("Key not found."))?;
            key.pk_sig
                .verify(&v.big_z.hash::<Sha256>(), &v.sig)
                .map_err(|_| anyhow!("Failed to verify signature."))
        })?;

        let mut a_z_pairs = PairingOutput::default();
        let mut c_y_pairs = PairingOutput::default();
        let mut c_tot = G1Projective::default();

        for v in signature.lams.iter() {
            a_z_pairs += Bls12_381::pairing(v.big_a, v.big_z);
            c_y_pairs += Bls12_381::pairing(v.big_c, pks.get(&v.client_id).unwrap().big_y);
            c_tot += v.big_c
        }

        let big_r_pair = Bls12_381::pairing(signature.big_r, self.g2);
        let big_s_pair = Bls12_381::pairing(self.g1, signature.big_s);

        let mut tags_scale_part = PairingOutput::default();

        for pk in pks.values() {
            tags_scale_part += pk.hs[0];
        }

        let mut msg_part = G1Projective::default();
        for (i, message) in messages.iter().enumerate() {
            msg_part += self.big_hs[i] * message
        }

        let p2 = tags_scale_part + c_y_pairs + big_r_pair;
        let p3 = big_s_pair + Bls12_381::pairing(c_tot, self.g2);
        let p4 = Bls12_381::pairing(msg_part, self.g2);

        let e1 = a_z_pairs == p2;
        let e2 = p3 == p4;

        if e1 != e2 {
            println!("Verification Failed.");
            return Err(anyhow!("Verification Failed."));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random;

    #[test]
    fn test_single_signature() {
        let n = 2usize;
        let t = 2usize;

        let mkhs = Mkhs::setup(n, t);
        let id = random();
        let key = mkhs.generate_keys(id);

        let messages = vec![Fr::from(2), Fr::from(10)];

        let signature = mkhs.sign(&key.sk, &messages);
        let check = mkhs.verify(&HashMap::from([(id, key.pk)]), &messages, &signature);

        assert!(check.is_ok());
    }

    #[test]
    fn test_aggregated_signature() {
        let n = 2usize;
        let t = 2usize;

        let mkhs = Mkhs::setup(n, t);

        // First user
        let id = random();
        let key = mkhs.generate_keys(id);

        let messages = vec![Fr::from(2), Fr::from(10)];
        let signature = mkhs.sign(&key.sk, &messages);

        // Second user
        let id2 = random();
        let key2 = mkhs.generate_keys(id2);

        let messages2 = vec![Fr::from(2), Fr::from(10)];
        let signature2 = mkhs.sign(&key2.sk, &messages2);

        // Combined signatures
        let combined_messages: Vec<Fr> = messages
            .iter()
            .zip(messages2)
            .map(|(v, v2)| *v + v2)
            .collect();
        let combined_signatures = mkhs.eval(&[signature, signature2]);

        let check = mkhs.verify(
            &HashMap::from([(id, key.pk), (id2, key2.pk)]),
            &combined_messages,
            &combined_signatures,
        );
        println!("check: {:?}", check);
        assert!(check.is_ok());
    }
}
