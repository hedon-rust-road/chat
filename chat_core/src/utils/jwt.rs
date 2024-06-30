use crate::User;

use jwt_simple::prelude::*;

#[allow(unused)]
const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISS: &str = "chat_server";
const JWT_AUD: &str = "chat_web";

#[allow(unused)]
pub struct EncodingKey(Ed25519KeyPair);

pub struct DecodingKey(Ed25519PublicKey);

#[allow(unused)]
impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }

    pub fn sign(&self, user: impl Into<User>) -> Result<String, jwt_simple::Error> {
        let claims = Claims::with_custom_claims(user.into(), Duration::from_secs(JWT_DURATION));
        let claims = claims.with_issuer(JWT_ISS).with_audience(JWT_AUD);
        self.0.sign(claims)
    }
}

#[allow(unused)]
impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }

    pub fn verify(&self, token: &str) -> Result<User, jwt_simple::Error> {
        let opts = VerificationOptions {
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISS])),
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUD])),
            ..VerificationOptions::default()
        };

        let claims = self.0.verify_token::<User>(token, Some(opts))?;
        Ok(claims.custom)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn jwt_sign_verify_should_work() -> anyhow::Result<()> {
        // generate public ed25519 key with OpenSSL
        // openssl genpkey -algorithm ed25519 -out ./fixtures/encoding.pem
        // openssl pkey -in ./fixtures/encoding.pem -pubout -out ./fixtures/decoding.pem
        // refer to: https://stackoverflow.com/questions/72151697/generating-public-ed25519-key-with-openssl
        let encoding_pem = include_str!("../../fixtures/encoding.pem");
        let decoding_pem = include_str!("../../fixtures/decoding.pem");
        let ek = EncodingKey::load(encoding_pem)?;
        let dk = DecodingKey::load(decoding_pem)?;

        let user = User::new(1, "hedon", "hedon@example.com");

        let token = ek.sign(user.clone())?;
        let user_from_token = dk.verify(&token)?;

        assert_eq!(user, user_from_token);
        Ok(())
    }
}
