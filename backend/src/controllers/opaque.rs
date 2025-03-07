use argon2::Argon2;
use axum::Error;
use generic_array::GenericArray;
use opaque_ke::ciphersuite::CipherSuite;
use opaque_ke::errors::ProtocolError;
use opaque_ke::rand::rngs::OsRng;
use opaque_ke::rand::{CryptoRng, RngCore};
use opaque_ke::{
    ClientLogin, ClientLoginFinishParameters, ClientRegistration,
    ClientRegistrationFinishParameters, CredentialFinalization, CredentialFinalizationLen,
    CredentialRequest, CredentialRequestLen, CredentialResponse, CredentialResponseLen,
    RegistrationRequest, RegistrationRequestLen, RegistrationResponse, RegistrationResponseLen,
    RegistrationUpload, RegistrationUploadLen, ServerLogin, ServerLoginFinishResult,
    ServerLoginStartParameters, ServerLoginStartResult, ServerRegistration, ServerRegistrationLen,
    ServerSetup,
};
use serde::de::IntoDeserializer;
use serde::Serialize;
use std::collections::HashMap;

type PasswordFile = ServerRegistration<CS>;

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
    ) -> Result<GenericArray<u8, RegistrationResponseLen<CS>>, opaque_ke::errors::ProtocolError>
    {
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
    ) -> Result<(), opaque_ke::errors::ProtocolError> {
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
    ) -> Result<GenericArray<u8, CredentialResponseLen<CS>>, opaque_ke::errors::ProtocolError> {
        let password_file = self
            .users
            .get(&username)
            .ok_or(ProtocolError::InvalidLoginError)?; // TODO

        println!("{:?}", password_file);

        let record = ServerRegistration::<CS>::deserialize(password_file)?;

        let server_login_start_result = ServerLogin::start(
            &mut self.rng,
            &self.opaque_server_setup,
            Some(record),
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
    ) -> Result<(), opaque_ke::errors::ProtocolError> {
        let server_login_start_result = self
            .login_sessions
            .get(&username)
            .cloned()
            .ok_or(ProtocolError::InvalidLoginError)?; // TODO

        let credential_finalization =
            CredentialFinalization::deserialize(&credential_finalization_bytes)?;

        let server_login_finish_result = server_login_start_result
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

    const PASSWORD: &str = "salasana123";
    const USERNAME: &str = "john.doe@example.com";

    fn generic_array_to_hex<N: generic_array::ArrayLength<u8>>(
        arr: &GenericArray<u8, N>,
    ) -> String {
        encode(arr.as_slice()) // Converts bytes to a hex string
    }

    #[test]
    fn test_registration() {
        let mut test_server_rng = StdRng::from_seed([0u8; 32]);
        let mut test_client_rng = StdRng::from_seed([1u8; 32]);
        let mut opaque_controller = OpaqueController::new(test_server_rng);

        // Client inits registration
        let client_registration_start =
            ClientRegistration::<CS>::start(&mut test_client_rng, PASSWORD.as_bytes()).unwrap();
        let registration_request = client_registration_start.message.serialize();

        // Server inits registration
        let registration_response = opaque_controller
            .register_init(USERNAME.to_string(), registration_request)
            .unwrap();

        // Client finalizes registration
        let client_registration_finish = client_registration_start
            .state
            .finish(
                &mut test_client_rng,
                PASSWORD.as_bytes(),
                RegistrationResponse::deserialize(&registration_response).unwrap(),
                ClientRegistrationFinishParameters::default(),
            )
            .unwrap();
        let registration_finish = client_registration_finish.message.serialize();

        // Server finalizes registration
        opaque_controller
            .register_finish(USERNAME.to_string(), registration_finish)
            .unwrap();

        assert_eq!(
            "f4277a7e82bd5eaaf7c3d4d0f6d259aeb10371c927f69931721d51de3d77c93e",
            generic_array_to_hex(&registration_request)
        );
        assert_eq!(
            "10b9b3e12e78b7630c275d61c9c82d9edd82d24eceb91eb2a5e35e167dcca34778bec00c6a1fba8fccf444b7f1a7af3dc90a20cae8b20769d2e0059819c40503",
            generic_array_to_hex(&registration_response)
        );
        assert_eq!(
            "620b9523e88a09cfa48bba1dda237bffd4db67c8806391def6d1f0f542b4f25df6ef552c1fd6f3b152fffa4cda4f63cf699a6e306d66742d4277c97ad84d4c2ffdf47dd7aa2d8992f6fd12b19482d75a4dd1915f51da1248e62548c1f08d77b33b713e9f2aff1b587314ba32d65b90fdfb58a4b4783b18c099ef2a95397c4375fd82ed9cede97b0bcfb7a56a1bdfa6e9ab535f2b28f381539e645cf470292209140d8310cb2638faa1b600fff49c69c7511d1a6d78d2f0251031f724324ae00b",
            generic_array_to_hex(&registration_finish)
        );

        assert_eq!(
            "620b9523e88a09cfa48bba1dda237bffd4db67c8806391def6d1f0f542b4f25df6ef552c1fd6f3b152fffa4cda4f63cf699a6e306d66742d4277c97ad84d4c2ffdf47dd7aa2d8992f6fd12b19482d75a4dd1915f51da1248e62548c1f08d77b33b713e9f2aff1b587314ba32d65b90fdfb58a4b4783b18c099ef2a95397c4375fd82ed9cede97b0bcfb7a56a1bdfa6e9ab535f2b28f381539e645cf470292209140d8310cb2638faa1b600fff49c69c7511d1a6d78d2f0251031f724324ae00b",
            generic_array_to_hex(opaque_controller.users.get(USERNAME).unwrap())
        );
    }

    #[test]
    fn test_login() {
        let mut test_server_rng = StdRng::from_seed([2u8; 32]);
        let mut test_client_rng = StdRng::from_seed([3u8; 32]);
        let mut opaque_controller = OpaqueController::new(test_server_rng);
        opaque_controller.users.insert(
            USERNAME.to_string(),
            GenericArray::from_exact_iter([
                98, 11, 149, 35, 232, 138, 9, 207, 164, 139, 186, 29, 218, 35, 123, 255, 212, 219,
                103, 200, 128, 99, 145, 222, 246, 209, 240, 245, 66, 180, 242, 93, 246, 239, 85,
                44, 31, 214, 243, 177, 82, 255, 250, 76, 218, 79, 99, 207, 105, 154, 110, 48, 109,
                102, 116, 45, 66, 119, 201, 122, 216, 77, 76, 47, 253, 244, 125, 215, 170, 45, 137,
                146, 246, 253, 18, 177, 148, 130, 215, 90, 77, 209, 145, 95, 81, 218, 18, 72, 230,
                37, 72, 193, 240, 141, 119, 179, 59, 113, 62, 159, 42, 255, 27, 88, 115, 20, 186,
                50, 214, 91, 144, 253, 251, 88, 164, 180, 120, 59, 24, 192, 153, 239, 42, 149, 57,
                124, 67, 117, 253, 130, 237, 156, 237, 233, 123, 11, 207, 183, 165, 106, 27, 223,
                166, 233, 171, 83, 95, 43, 40, 243, 129, 83, 158, 100, 92, 244, 112, 41, 34, 9, 20,
                13, 131, 16, 203, 38, 56, 250, 161, 182, 0, 255, 244, 156, 105, 199, 81, 29, 26,
                109, 120, 210, 240, 37, 16, 49, 247, 36, 50, 74, 224, 11,
            ])
            .unwrap(),
        );

        let client_login_start =
            ClientLogin::<CS>::start(&mut test_client_rng, PASSWORD.as_bytes()).unwrap();
        let credential_request = client_login_start.message.serialize();

        let credential_response = opaque_controller
            .login_start(USERNAME.to_string(), credential_request)
            .unwrap();

        assert_eq!(
            "d0e855aef328eddd4934e9dc98027c985750fbb6a6c117297bbbdf415697e81422267398fec85f09efad2e42951d56a204f877bc6b678dee04a37b59cb5bc7e856a50937ab14dff8bb5b3c2da82eb757b6c5bdc7bd045fa6c83af7609a71b809", 
            generic_array_to_hex(&credential_request)
        );

        assert_eq!(
            "9825516c58ab90fa5eeee174203e3618dca5499214b7f937b79d1aad08ea321471fd9e51dfd366375a83e13311636cba849533b2a4372243d7ef42b6cfed6ec004a391900398ccac5cb15c81f39d98aef47be0eeef4ce99a361c28ae5ece95a5220f1b8f9478f04b7f217a98701ebb5c032548afddb33fcf04b83f9f54fff508e146a33a755ce91a8ede7ed61065184c970070ea0d8626d046ae12909bca0875fa9c91d45b7c676f7a00840f5eaef340eb0609109b931ae54544b1cfc4f1006f003889a079646a9be8ba4fff459af7a94a250b7bfe16544a97d28ca58033b2b3bc8f3490a739e8a7449fd7f395b01ea6e8fa7fb86df0f36dfd2e3b9e29e78a0645de0ae3589461cb7a15504d716b597dc0d9c7b099f8742accb22a60da9d01ef020511e682a5d3227f1c1087e6ce3c41d5549e99826c942b95dc293835942535", 
            generic_array_to_hex(&credential_response)
        );

        // println!("{:?}", opaque_controller.login_sessions);
        // println!("{:?}", opaque_controller.users);

        let client_login_finish = client_login_start
            .state
            .finish(
                PASSWORD.as_bytes(),
                CredentialResponse::deserialize(&credential_response).unwrap(),
                ClientLoginFinishParameters::default(),
            )
            .unwrap(); // panics here

        let credential_finalization_bytes = client_login_finish.message.serialize();

        // Client sends credential_finalization_bytes to server

        opaque_controller
            .login_finish(USERNAME.to_string(), credential_finalization_bytes)
            .unwrap();
    }

    fn account_registration<R: RngCore + CryptoRng>(
        server_setup: &ServerSetup<CS>,
        username: String,
        password: String,
        mut client_rng: R,
        mut server_rng: R,
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

    fn account_login<R: RngCore + CryptoRng>(
        server_setup: &ServerSetup<CS>,
        username: String,
        password: String,
        password_file_bytes: &[u8],
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

        assert_eq!(
            "d0e855aef328eddd4934e9dc98027c985750fbb6a6c117297bbbdf415697e81422267398fec85f09efad2e42951d56a204f877bc6b678dee04a37b59cb5bc7e856a50937ab14dff8bb5b3c2da82eb757b6c5bdc7bd045fa6c83af7609a71b809", 
            generic_array_to_hex(&credential_request_bytes)
        );

        // assert_eq!(
        //     "9825516c58ab90fa5eeee174203e3618dca5499214b7f937b79d1aad08ea321471fd9e51dfd366375a83e13311636cba849533b2a4372243d7ef42b6cfed6ec004a391900398ccac5cb15c81f39d98aef47be0eeef4ce99a361c28ae5ece95a5220f1b8f9478f04b7f217a98701ebb5c032548afddb33fcf04b83f9f54fff508e146a33a755ce91a8ede7ed61065184c970070ea0d8626d046ae12909bca0875fa9c91d45b7c676f7a00840f5eaef340eb0609109b931ae54544b1cfc4f1006f003889a079646a9be8ba4fff459af7a94a250b7bfe16544a97d28ca58033b2b3bc8f3490a739e8a7449fd7f395b01ea6e8fa7fb86df0f36dfd2e3b9e29e78a0645de0ae3589461cb7a15504d716b597dc0d9c7b099f8742accb22a60da9d01ef020511e682a5d3227f1c1087e6ce3c41d5549e99826c942b95dc293835942535",
        //     generic_array_to_hex(&credential_response_bytes)
        // );

        client_login_finish_result.session_key == server_login_finish_result.session_key
    }

    #[test]
    fn test_account_registration() {
        let mut server_rng = StdRng::from_seed([0u8; 32]);
        let mut client_rng = StdRng::from_seed([1u8; 32]);
        let mut server_setup = ServerSetup::<CS>::new(&mut server_rng);

        let server_registration = account_registration(
            &server_setup,
            USERNAME.to_string(),
            PASSWORD.to_string(),
            client_rng,
            server_rng,
        );

        assert_eq!(
            "620b9523e88a09cfa48bba1dda237bffd4db67c8806391def6d1f0f542b4f25df6ef552c1fd6f3b152fffa4cda4f63cf699a6e306d66742d4277c97ad84d4c2ffdf47dd7aa2d8992f6fd12b19482d75a4dd1915f51da1248e62548c1f08d77b33b713e9f2aff1b587314ba32d65b90fdfb58a4b4783b18c099ef2a95397c4375fd82ed9cede97b0bcfb7a56a1bdfa6e9ab535f2b28f381539e645cf470292209140d8310cb2638faa1b600fff49c69c7511d1a6d78d2f0251031f724324ae00b",
            generic_array_to_hex(&server_registration)
        );
    }

    #[test]
    fn my_test() {
        let mut server_rng = StdRng::from_seed([0u8; 32]);
        let mut client_rng = StdRng::from_seed([1u8; 32]);
        let mut server_setup = ServerSetup::<CS>::new(&mut server_rng);
    }

    #[test]
    fn test_example() {
        let mut server_rng = StdRng::from_seed([0u8; 32]);
        let mut client_rng = StdRng::from_seed([1u8; 32]);
        let mut server_setup = ServerSetup::<CS>::new(&mut server_rng);
        let server_registration = account_registration(
            &server_setup,
            USERNAME.to_string(),
            PASSWORD.to_string(),
            client_rng,
            server_rng,
        );

        let mut server_rng2 = StdRng::from_seed([2u8; 32]);
        let mut client_rng2 = StdRng::from_seed([3u8; 32]);
        let mut server_setup2 = ServerSetup::<CS>::new(&mut server_rng2);
        println!("{:?}", server_registration);
        println!("{:?}", generic_array_to_hex(&server_registration));

        account_login(
            &server_setup2,
            USERNAME.to_string(),
            PASSWORD.to_string(),
            &server_registration,
            client_rng2,
            server_rng2,
        );
    }
}
