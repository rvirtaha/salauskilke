use argon2::Argon2;
use generic_array::GenericArray;
use opaque_ke::ciphersuite::CipherSuite;
use opaque_ke::errors::ProtocolError;
use opaque_ke::rand::rngs::OsRng;
use opaque_ke::rand::{CryptoRng, RngCore};
use opaque_ke::{
    CredentialFinalization, CredentialFinalizationLen, CredentialRequest, CredentialRequestLen,
    CredentialResponseLen, RegistrationRequest, RegistrationRequestLen, RegistrationResponseLen,
    RegistrationUpload, RegistrationUploadLen, ServerLogin, ServerLoginStartParameters,
    ServerLoginStartResult, ServerRegistration, ServerRegistrationLen, ServerSetup,
};
use std::collections::HashMap;

use super::errors::ServiceError;

pub struct CS;

impl CipherSuite for CS {
    type OprfCs = opaque_ke::Ristretto255;
    type KeGroup = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDh;
    type Ksf = Argon2<'static>;
}

pub struct OpaqueController<R: RngCore + CryptoRng> {
    opaque_server_setup: ServerSetup<CS>,
    users: HashMap<String, GenericArray<u8, ServerRegistrationLen<CS>>>,
    rng: R,
    login_sessions: HashMap<String, ServerLoginStartResult<CS>>,
}

impl Default for OpaqueController<OsRng> {
    fn default() -> Self {
        OpaqueController::new(OsRng)
    }
}

impl<R: RngCore + CryptoRng> OpaqueController<R> {
    pub fn new(mut rng: R) -> Self {
        Self {
            opaque_server_setup: ServerSetup::<CS>::new(&mut rng),
            users: HashMap::new(),
            rng,
            login_sessions: HashMap::new(),
        }
    }

    pub fn register_init(
        &mut self,
        username: String,
        registration_request: GenericArray<u8, RegistrationRequestLen<CS>>,
    ) -> Result<GenericArray<u8, RegistrationResponseLen<CS>>, ServiceError> {
        let server_registration_start_result = ServerRegistration::<CS>::start(
            &self.opaque_server_setup,
            RegistrationRequest::deserialize(&registration_request)?,
            username.as_bytes(),
        )?;

        Ok(server_registration_start_result.message.serialize())
    }

    pub fn register_finish(
        &mut self,
        username: String,
        registration_finish: GenericArray<u8, RegistrationUploadLen<CS>>,
    ) -> Result<(), ServiceError> {
        let password_file = ServerRegistration::finish(RegistrationUpload::<CS>::deserialize(
            &registration_finish,
        )?);
        let serialize_password_file = password_file.serialize();

        self.users.insert(username, serialize_password_file);

        Ok(())
    }

    pub fn login_start(
        &mut self,
        username: String,
        credential_request: GenericArray<u8, CredentialRequestLen<CS>>,
    ) -> Result<GenericArray<u8, CredentialResponseLen<CS>>, ServiceError> {
        let record = self
            .users
            .get(&username)
            .map(|p| ServerRegistration::<CS>::deserialize(p))
            .transpose()?;

        let server_login_start_result = ServerLogin::start(
            &mut self.rng,
            &self.opaque_server_setup,
            record,
            CredentialRequest::deserialize(&credential_request)?,
            username.as_bytes(),
            ServerLoginStartParameters::default(),
        )?;

        self.login_sessions
            .insert(username.clone(), server_login_start_result.clone());

        Ok(server_login_start_result.message.serialize())
    }

    pub fn login_finish(
        &mut self,
        username: String,
        credential_finalization_bytes: GenericArray<u8, CredentialFinalizationLen<CS>>,
    ) -> Result<(), ServiceError> {
        let server_login_start_result = self
            .login_sessions
            .get(&username)
            .cloned()
            .ok_or(ServiceError::LoginSessionMissingOrExpired)?;

        let credential_finalization =
            CredentialFinalization::deserialize(&credential_finalization_bytes)?;

        server_login_start_result
            .state
            .finish(credential_finalization)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]

    use super::*;
    use hex::encode;
    use opaque_ke::rand::rngs::StdRng;
    use opaque_ke::rand::SeedableRng;
    use opaque_ke::{
        ClientLogin, ClientLoginFinishParameters, ClientRegistration,
        ClientRegistrationFinishParameters, CredentialResponse, RegistrationResponse,
    };

    const PASSWORD: &str = "salasana123";
    const USERNAME: &str = "john.doe@example.com";

    fn generic_array_to_hex<N: generic_array::ArrayLength<u8>>(
        arr: &GenericArray<u8, N>,
    ) -> String {
        encode(arr.as_slice()) // Converts bytes to a hex string
    }

    fn register<R: RngCore + CryptoRng>(
        username: String,
        password: String,
        mut client_rng: R,
        server_rng: R,
    ) -> GenericArray<u8, ServerRegistrationLen<CS>> {
        let mut opaque_controller = OpaqueController::new(server_rng);

        // Client inits registration
        let client_registration_start =
            ClientRegistration::<CS>::start(&mut client_rng, password.as_bytes()).unwrap();
        let registration_request = client_registration_start.message.serialize();

        // Server inits registration
        let registration_response = opaque_controller
            .register_init(username.clone(), registration_request)
            .unwrap();

        // Client finalizes registration
        let client_registration_finish = client_registration_start
            .state
            .finish(
                &mut client_rng,
                password.as_bytes(),
                RegistrationResponse::deserialize(&registration_response).unwrap(),
                ClientRegistrationFinishParameters::default(),
            )
            .unwrap();
        let password_file = client_registration_finish.message.serialize();

        // Server finalizes registration
        opaque_controller
            .register_finish(username, password_file)
            .unwrap();

        password_file
    }

    fn login<R: RngCore + CryptoRng>(
        username: String,
        password: String,
        password_file_bytes: GenericArray<u8, ServerRegistrationLen<CS>>,
        mut client_rng: R,
        server_rng: R,
    ) {
        let mut opaque_controller = OpaqueController::new(server_rng);
        opaque_controller
            .users
            .insert(username.clone(), password_file_bytes);

        // Client start login
        let client_login_start =
            ClientLogin::<CS>::start(&mut client_rng, password.as_bytes()).unwrap();
        let credential_request = client_login_start.message.serialize();

        // Server start login
        let credential_response = opaque_controller
            .login_start(username.clone(), credential_request)
            .unwrap();

        // Client finish login
        let client_login_finish = client_login_start
            .state
            .finish(
                password.as_bytes(),
                CredentialResponse::deserialize(&credential_response).unwrap(),
                ClientLoginFinishParameters::default(),
            )
            .unwrap(); // panics here

        let credential_finalization_bytes = client_login_finish.message.serialize();

        // Client sends credential_finalization_bytes to server

        opaque_controller
            .login_finish(username, credential_finalization_bytes)
            .unwrap();

        assert_eq!(
            "7227e3a1ca503c4a69784253ec3b3dcc1cf04be53b671b0fc7dd8bf18510ab447898a8f25aa9ebc9865ba7e7ef3b41bbe8f4413f532e223197160e1b7f750fd7",
            generic_array_to_hex(&client_login_finish.session_key)
        );
        assert_eq!(
            "90ff2ccfcc7dc97c038ac08198a69cf3106534343687bda5eef2d9587b6153302dacb886df251da4db26d42d8c576653ca54be075fb103f52826bcd62ef0b49d",
            generic_array_to_hex(&client_login_finish.export_key)
        );
    }

    fn example_register<R: RngCore + CryptoRng>(
        username: String,
        password: String,
        server_setup: &ServerSetup<CS>,
        mut client_rng: R,
    ) -> GenericArray<u8, ServerRegistrationLen<CS>> {
        let client_registration_start_result =
            ClientRegistration::<CS>::start(&mut client_rng, password.as_bytes()).unwrap();
        let registration_request_bytes = client_registration_start_result.message.serialize();

        // Client sends registration_request_bytes to server

        let server_registration_start_result = ServerRegistration::<CS>::start(
            server_setup,
            RegistrationRequest::deserialize(&registration_request_bytes).unwrap(),
            username.as_bytes(),
        )
        .unwrap();
        let registration_response_bytes = server_registration_start_result.message.serialize();

        // Server sends registration_response_bytes to client

        let client_finish_registration_result = client_registration_start_result
            .state
            .finish(
                &mut client_rng,
                password.as_bytes(),
                RegistrationResponse::deserialize(&registration_response_bytes).unwrap(),
                ClientRegistrationFinishParameters::default(),
            )
            .unwrap();
        let message_bytes = client_finish_registration_result.message.serialize();

        // Client sends message_bytes to server

        let password_file = ServerRegistration::finish(
            RegistrationUpload::<CS>::deserialize(&message_bytes).unwrap(),
        )
        .serialize();

        assert_eq!(
            "f4277a7e82bd5eaaf7c3d4d0f6d259aeb10371c927f69931721d51de3d77c93e",
            generic_array_to_hex(&registration_request_bytes)
        );
        assert_eq!(
            "10b9b3e12e78b7630c275d61c9c82d9edd82d24eceb91eb2a5e35e167dcca34778bec00c6a1fba8fccf444b7f1a7af3dc90a20cae8b20769d2e0059819c40503",
            generic_array_to_hex(&registration_response_bytes)
        );
        assert_eq!(
            "620b9523e88a09cfa48bba1dda237bffd4db67c8806391def6d1f0f542b4f25df6ef552c1fd6f3b152fffa4cda4f63cf699a6e306d66742d4277c97ad84d4c2ffdf47dd7aa2d8992f6fd12b19482d75a4dd1915f51da1248e62548c1f08d77b33b713e9f2aff1b587314ba32d65b90fdfb58a4b4783b18c099ef2a95397c4375fd82ed9cede97b0bcfb7a56a1bdfa6e9ab535f2b28f381539e645cf470292209140d8310cb2638faa1b600fff49c69c7511d1a6d78d2f0251031f724324ae00b",
            generic_array_to_hex(&password_file)
        );

        password_file
    }

    fn example_login<R: RngCore + CryptoRng>(
        username: String,
        password: String,
        password_file_bytes: &[u8],
        server_setup: &ServerSetup<CS>,
        mut client_rng: R,
        mut server_rng: R,
    ) -> bool {
        let client_login_start_result =
            ClientLogin::<CS>::start(&mut client_rng, password.as_bytes()).unwrap();
        let credential_request_bytes = client_login_start_result.message.serialize();

        // Client sends credential_request_bytes to server

        let password_file = ServerRegistration::<CS>::deserialize(password_file_bytes).unwrap();
        let server_login_start_result = ServerLogin::start(
            &mut server_rng,
            server_setup,
            Some(password_file),
            CredentialRequest::deserialize(&credential_request_bytes).unwrap(),
            username.as_bytes(),
            ServerLoginStartParameters::default(),
        )
        .unwrap();
        let credential_response_bytes = server_login_start_result.message.serialize();

        // Server sends credential_response_bytes to client

        let result = client_login_start_result.state.finish(
            password.as_bytes(),
            CredentialResponse::deserialize(&credential_response_bytes).unwrap(),
            ClientLoginFinishParameters::default(),
        );

        if result.is_err() {
            // Client-detected login failure
            return false;
        }
        let client_login_finish_result = result.unwrap();
        let credential_finalization_bytes = client_login_finish_result.message.serialize();

        // Client sends credential_finalization_bytes to server

        let server_login_finish_result = server_login_start_result
            .state
            .finish(CredentialFinalization::deserialize(&credential_finalization_bytes).unwrap())
            .unwrap();

        client_login_finish_result.session_key == server_login_finish_result.session_key
    }

    #[test]
    fn my_test() {
        let server_rng = StdRng::from_seed([0u8; 32]);
        let client_rng = StdRng::from_seed([1u8; 32]);

        let password_file = register(
            USERNAME.to_string(),
            PASSWORD.to_string(),
            client_rng.clone(),
            server_rng.clone(),
        );

        assert_eq!(
            "620b9523e88a09cfa48bba1dda237bffd4db67c8806391def6d1f0f542b4f25df6ef552c1fd6f3b152fffa4cda4f63cf699a6e306d66742d4277c97ad84d4c2ffdf47dd7aa2d8992f6fd12b19482d75a4dd1915f51da1248e62548c1f08d77b33b713e9f2aff1b587314ba32d65b90fdfb58a4b4783b18c099ef2a95397c4375fd82ed9cede97b0bcfb7a56a1bdfa6e9ab535f2b28f381539e645cf470292209140d8310cb2638faa1b600fff49c69c7511d1a6d78d2f0251031f724324ae00b", 
            generic_array_to_hex(&password_file)
        );

        login(
            USERNAME.to_string(),
            PASSWORD.to_string(),
            password_file,
            client_rng,
            server_rng,
        );
    }

    #[test]
    fn test_example() {
        let mut server_rng = StdRng::from_seed([0u8; 32]);
        let client_rng = StdRng::from_seed([1u8; 32]);
        let server_setup = ServerSetup::<CS>::new(&mut server_rng);

        let server_registration = example_register(
            USERNAME.to_string(),
            PASSWORD.to_string(),
            &server_setup,
            client_rng.clone(),
        );

        assert_eq!(
            "620b9523e88a09cfa48bba1dda237bffd4db67c8806391def6d1f0f542b4f25df6ef552c1fd6f3b152fffa4cda4f63cf699a6e306d66742d4277c97ad84d4c2ffdf47dd7aa2d8992f6fd12b19482d75a4dd1915f51da1248e62548c1f08d77b33b713e9f2aff1b587314ba32d65b90fdfb58a4b4783b18c099ef2a95397c4375fd82ed9cede97b0bcfb7a56a1bdfa6e9ab535f2b28f381539e645cf470292209140d8310cb2638faa1b600fff49c69c7511d1a6d78d2f0251031f724324ae00b", 
            generic_array_to_hex(&server_registration)
        );

        assert!(example_login(
            USERNAME.to_string(),
            PASSWORD.to_string(),
            &server_registration,
            &server_setup,
            client_rng,
            server_rng,
        ));
    }
}
