use std::sync::Arc;

use bb8_postgres::{bb8::Pool, PostgresConnectionManager};
use shaku::module;
use source_control_domain::factories::platform_account::PlatformAccountFactoryImpl;
use source_control_event_store_persistence_adapter::{
    provider::{EventStoreProviderImpl, EventStoreProviderImplParameters},
    repositories::organization_repository::OrganizationRepositoryImpl,
};
use source_control_postgres_persistence_adapter::{
    projectors::organization::OrganizationProjector,
    provider::{PostgresProviderImpl, PostgresProviderImplParameters},
    queries::get_organizations::GetOrganizationsQueryHandlerImpl,
};
use tokio_postgres::NoTls;

use crate::{
    commands::{
        add_platform_account::AddPlatformAccountCommandHandlerImpl,
        create_organization::CreateOrganizationCommandHandlerImpl,
        remove_platform_account::RemovePlatformAccountCommandHandlerImpl,
    },
    queries::{
        get_organization::GetOrganizationQueryHandlerImpl,
        get_organization_log::GetOrganizationLogQueryHandlerImpl,
    },
};

module! {
    pub ApplicationModule {
        components = [
            PostgresProviderImpl,
            EventStoreProviderImpl
        ],
        providers = [
            AddPlatformAccountCommandHandlerImpl,
            OrganizationRepositoryImpl,
            PlatformAccountFactoryImpl,
            CreateOrganizationCommandHandlerImpl,
            GetOrganizationLogQueryHandlerImpl,
            GetOrganizationQueryHandlerImpl,
            RemovePlatformAccountCommandHandlerImpl,
            OrganizationProjector,
            GetOrganizationsQueryHandlerImpl,
        ],
    }
}

pub fn get_module(
    postgres_client: Arc<Pool<PostgresConnectionManager<NoTls>>>,
    eventstore_client: Arc<eventstore::Client>,
) -> ApplicationModule {
    ApplicationModule::builder()
        .with_component_parameters::<PostgresProviderImpl>(PostgresProviderImplParameters {
            client: postgres_client,
        })
        .with_component_parameters::<EventStoreProviderImpl>(EventStoreProviderImplParameters {
            client: eventstore_client,
        })
        .build()
}
