use beep_auth::KeycloakAuthRepository;
use communities_core::CommunitiesService;

#[derive(Clone)]
pub struct AuthState {
    keycloak: KeycloakAuthRepository,
    service: CommunitiesService,
}

impl AuthState {
    pub fn new(keycloak: KeycloakAuthRepository, service: CommunitiesService) -> Self {
        Self { keycloak, service }
    }

    pub fn keycloak(&self) -> KeycloakAuthRepository {
        self.keycloak.clone()
    }

    pub fn service(&self) -> CommunitiesService {
        self.service.clone()
    }
}
