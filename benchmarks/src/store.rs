use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU64},
        Arc,
    },
};

use rand::Rng;
use tokio::{
    sync::{
        mpsc::{
            channel,
            error::{SendError, TryRecvError},
            Receiver, Sender,
        },
        Mutex, RwLock,
    },
    task::JoinHandle,
};

pub struct Store {
    sender: Arc<Sender<Option<StoreActions>>>,
    receiver: Arc<Mutex<Receiver<Option<StoreActions>>>>,
    open: AtomicBool,
    message_count: AtomicU64,
    handled_count: AtomicU64,
    listen_future: RwLock<Option<JoinHandle<Result<(), ()>>>>,

    read_copy: Arc<RwLock<StoreData>>,
    write_copy: Arc<RwLock<StoreData>>,
}

struct StoreData {
    organizations: HashMap<u64, Organization>,
    organization_ids: Vec<u64>,
    platform_accounts: HashMap<u64, (u64, usize)>,
    platform_account_ids: Vec<u64>,
}

impl Store {
    pub fn new() -> Self {
        let (tx, rx): (Sender<Option<StoreActions>>, Receiver<Option<StoreActions>>) =
            channel(20000);
        Store {
            sender: Arc::new(tx),
            receiver: Arc::new(Mutex::new(rx)),
            message_count: AtomicU64::new(0),
            handled_count: AtomicU64::new(0),
            listen_future: RwLock::new(None),
            open: AtomicBool::new(true),
            read_copy: Arc::new(RwLock::new(StoreData {
                organization_ids: vec![],
                organizations: HashMap::new(),
                platform_accounts: HashMap::new(),
                platform_account_ids: vec![],
            })),
            write_copy: Arc::new(RwLock::new(StoreData {
                organization_ids: vec![],
                organizations: HashMap::new(),
                platform_accounts: HashMap::new(),
                platform_account_ids: vec![],
            })),
        }
    }

    pub async fn get_random_platform_account(&self) -> (u64, u64) {
        let store = self.read_copy.read().await;
        let pids = &store.platform_account_ids;
        let pas = &store.platform_accounts;
        if pids.is_empty() {
            return (0, 0); // Or handle the empty case appropriately
        }

        let mut rng = rand::rng(); // Use thread_rng for better performance
        let index = rng.random_range(0..pids.len());
        let platform_account_id = pids.get(index).unwrap();
        let organization_id = pas.get(platform_account_id).unwrap();

        (organization_id.0, *platform_account_id)
    }

    pub async fn get_random_organization(&self) -> Organization {
        let store = self.read_copy.read().await;
        let ids = &store.organization_ids;
        if ids.is_empty() {
            return Organization {
                id: 0,
                platform_accounts: Vec::new(),
            }; // Or handle the empty case appropriately
        }

        let mut rng = rand::rng();
        let index = rng.random_range(0..ids.len());
        let id = ids[index];

        store.organizations.get(&id).unwrap().clone()
    }

    pub async fn get_missing_organizations(&self, organizations: Vec<u64>) -> Vec<u64> {
        let r = self.read_copy.read().await;

        let mut res = vec![];
        for organization in organizations {
            if r.organizations.contains_key(&organization) {
                continue;
            }

            res.push(organization);
        }

        res
    }

    pub async fn has_organization(&self, organization: u64) -> bool {
        let r = self.read_copy.read().await;
        r.organizations.contains_key(&organization)
    }

    pub async fn add_action(
        &self,
        action: StoreActions,
    ) -> Result<(), SendError<Option<StoreActions>>> {
        let open = self.open.load(std::sync::atomic::Ordering::Relaxed);
        if open {
            return match self.sender.send(Some(action)).await {
                Ok(_) => {
                    self.message_count
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    Ok(())
                }
                Err(err) => Err(err),
            };
        }

        Err(SendError(Some(action)))
    }

    async fn listen(&self) -> Result<(), ()> {
        let receiver_cloned = self.receiver.clone();
        let mut receiver = match receiver_cloned.try_lock() {
            Ok(val) => Ok(val),
            Err(_) => Err(()),
        }?;
        loop {
            let mut buffer = Vec::new();
            let result = receiver.recv().await.unwrap_or(None);
            buffer.push(result);

            for _ in 0..100 {
                match receiver.try_recv() {
                    Ok(ok) => buffer.push(ok),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => buffer.push(None),
                };
            }

            if buffer.is_empty() {
                return Ok(());
            }

            let mut done: bool = false;

            // Apply the changes to the write copy
            let mut write_copy = self.write_copy.write().await;
            done |= Store::apply_actions(&mut write_copy, &buffer);

            // Swap the read and write copies
            let mut read_copy = self.read_copy.write().await;
            std::mem::swap(&mut *write_copy, &mut *read_copy);

            // Unlock the read_copy
            std::mem::drop(read_copy);

            // Apply the changes to the old read copy, that is now the write copy
            done |= Store::apply_actions(&mut write_copy, &buffer);
            // Unlock the write_copy
            std::mem::drop(write_copy);

            self.handled_count
                .fetch_add(buffer.len() as u64, std::sync::atomic::Ordering::Relaxed);

            if done {
                return Ok(());
            }
        }
    }

    pub async fn run(self: &Arc<Self>) {
        let self_clone = self.clone();
        let run = tokio::spawn(async move { self_clone.listen().await });

        let mut a = self.listen_future.write().await;
        *a = Some(run);
    }

    pub async fn close(&self) -> Result<(), SendError<Option<StoreActions>>> {
        self.open.store(false, std::sync::atomic::Ordering::Relaxed);
        self.sender.send(None).await?;

        let mut listen_future = self.listen_future.write().await;
        if let Some(join_handle) = listen_future.take() {
            let res = join_handle.await;
            if res.is_err() {
                return Err(SendError(None));
            }
        }

        Ok(())
    }

    fn remove_platform_account(
        platform_accounts: &mut HashMap<u64, (u64, usize)>,
        platform_account_ids: &mut Vec<u64>,
        platform_account: u64,
    ) {
        let idx = platform_accounts.get(&platform_account);
        if let Some(idx) = idx {
            let id = *idx;
            platform_accounts.remove(&platform_account);

            if platform_account_ids.len() > 1 && id.1 != platform_account_ids.len() - 1 {
                let last_value = platform_account_ids.pop().unwrap();
                let last_org = *platform_accounts.get(&last_value).unwrap();
                platform_account_ids[id.1] = last_value;
                platform_accounts.insert(last_value, (last_org.0, id.1));
            } else {
                platform_account_ids.remove(id.1);
            }
        }
    }

    fn add_platform_account(
        platform_accounts: &mut HashMap<u64, (u64, usize)>,
        platform_account_ids: &mut Vec<u64>,
        platform_account: u64,
        organization: u64,
    ) {
        let index = platform_account_ids.len();
        platform_account_ids.push(platform_account);
        platform_accounts.insert(platform_account, (organization, index));
    }

    fn apply_action(data: &mut StoreData, action: &Option<StoreActions>) -> bool {
        let organizations = &mut data.organizations;
        let ids = &mut data.organization_ids;
        let pas = &mut data.platform_accounts;
        let pids = &mut data.platform_account_ids;

        match &action {
            Some(StoreActions::AddOrganization {
                organization_id,
                platform_accounts,
            }) => {
                let existing = organizations.get_mut(organization_id);
                if let Some(existing_org) = existing {
                    if !platform_accounts.is_empty() {
                        for pa in &existing_org.platform_accounts {
                            Store::remove_platform_account(pas, pids, *pa);
                        }

                        existing_org.platform_accounts = platform_accounts.clone();

                        for pa in platform_accounts {
                            Store::add_platform_account(pas, pids, *pa, *organization_id);
                        }
                    }
                    return false;
                }

                for pa in platform_accounts {
                    Store::add_platform_account(pas, pids, *pa, *organization_id);
                }

                let organization = Organization {
                    id: *organization_id,
                    platform_accounts: platform_accounts.clone(),
                };
                organizations.insert(*organization_id, organization);
                ids.push(*organization_id);

                false
            }
            Some(StoreActions::AddPlatformAccount {
                organization_id,
                platform_account_id,
            }) => {
                let existing = organizations.get_mut(organization_id);
                if let Some(existing_org) = existing {
                    existing_org.platform_accounts.push(*platform_account_id);
                    Store::add_platform_account(pas, pids, *platform_account_id, *organization_id);
                    return false;
                }

                println!("Organization not found while adding platform account");
                false
            }
            Some(StoreActions::DeletePlatformAccount {
                organization_id,
                platform_account_id,
            }) => {
                let existing = organizations.get_mut(organization_id);
                if let Some(existing_org) = existing {
                    let index = existing_org
                        .platform_accounts
                        .iter()
                        .position(|n| *n == *platform_account_id);
                    if let Some(idx) = index {
                        existing_org.platform_accounts.remove(idx);
                        Store::remove_platform_account(pas, pids, *platform_account_id);
                        return false;
                    }
                    println!("Platform Account not found while deleting platform account");

                    return false;
                }

                println!("Organization not found while deleting platform account");
                false
            }
            None => true,
        }
    }

    fn apply_actions(data: &mut StoreData, actions: &[Option<StoreActions>]) -> bool {
        let mut done = false;
        for action in actions {
            let res = Store::apply_action(data, action);
            done |= res;
        }

        done
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct Organization {
    pub id: u64,
    pub platform_accounts: Vec<u64>,
}

pub enum StoreActions {
    AddOrganization {
        organization_id: u64,
        platform_accounts: Vec<u64>,
    },
    AddPlatformAccount {
        organization_id: u64,
        platform_account_id: u64,
    },
    DeletePlatformAccount {
        organization_id: u64,
        platform_account_id: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::ThreadRng;
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_add_organization() {
        let store = Arc::new(Store::new()); // Wrap in Arc
        let org_id = 123;
        let platform_accounts = vec![1, 2, 3];

        // Spawn the run task
        store.run().await;

        let res = store
            .add_action(StoreActions::AddOrganization {
                organization_id: org_id,
                platform_accounts: platform_accounts.clone(),
            })
            .await;

        assert!(res.is_ok());

        // Give the store some time to process the action
        let res = store.close().await;
        assert!(res.is_ok());

        let org = store.get_random_organization().await;
        assert_eq!(org.id, org_id);
        assert_eq!(org.platform_accounts, platform_accounts);

        let data = store.read_copy.read().await;
        let ids = &data.organization_ids;

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], org_id);
    }

    #[tokio::test]
    async fn test_add_platform_account() {
        let store = Arc::new(Store::new());
        let org_id = 456;
        let initial_accounts = vec![4, 5];
        let new_account = 6;

        // Spawn the run task
        store.run().await;

        // First, add the organization
        let res = store
            .add_action(StoreActions::AddOrganization {
                organization_id: org_id,
                platform_accounts: initial_accounts.clone(),
            })
            .await;
        assert!(res.is_ok());

        // Then, add the platform account
        let res = store
            .add_action(StoreActions::AddPlatformAccount {
                organization_id: org_id,
                platform_account_id: new_account,
            })
            .await;
        assert!(res.is_ok());
        let res = store.close().await;
        assert!(res.is_ok());

        let org = store.get_random_organization().await;
        let mut expected_accounts = initial_accounts.clone();
        expected_accounts.push(new_account);
        assert_eq!(org.platform_accounts, expected_accounts);
    }

    #[tokio::test]
    async fn test_delete_platform_account() {
        let store = Arc::new(Store::new());
        let org_id = 789;
        let initial_accounts = vec![7, 8, 9];
        let account_to_delete = 8;

        // Spawn the run task
        store.run().await;

        // First, add the organization
        let res = store
            .add_action(StoreActions::AddOrganization {
                organization_id: org_id,
                platform_accounts: initial_accounts.clone(),
            })
            .await;
        assert!(res.is_ok());

        // Then, delete the platform account
        let res = store
            .add_action(StoreActions::DeletePlatformAccount {
                organization_id: org_id,
                platform_account_id: account_to_delete,
            })
            .await;
        assert!(res.is_ok());

        let res = store.close().await;
        assert!(res.is_ok());

        let org = store.get_random_organization().await;
        let expected_accounts: Vec<u64> = initial_accounts
            .into_iter()
            .filter(|&x| x != account_to_delete)
            .collect();
        assert_eq!(org.platform_accounts, expected_accounts);
    }

    #[tokio::test]
    async fn test_get_random_platform_account() {
        let store = Arc::new(Store::new());
        let org_id = 101112;
        let platform_accounts = vec![10, 11, 12];

        // Spawn the run task
        store.run().await;

        // Add the organization
        let res = store
            .add_action(StoreActions::AddOrganization {
                organization_id: org_id,
                platform_accounts: platform_accounts.clone(),
            })
            .await;
        assert!(res.is_ok());
        let res = store.close().await;
        assert!(res.is_ok());

        // Get a random platform account
        let (returned_org_id, returned_account_id) = store.get_random_platform_account().await;
        assert_eq!(returned_org_id, org_id);
        assert!(platform_accounts.contains(&returned_account_id));
    }

    #[tokio::test]
    async fn test_multiple_organizations() {
        let store = Arc::new(Store::new());
        let org_id1 = 1;
        let org_id2 = 2;
        let accounts1 = vec![1, 2];
        let accounts2 = vec![3, 4];

        // Spawn the run task
        store.run().await;

        let res = store
            .add_action(StoreActions::AddOrganization {
                organization_id: org_id1,
                platform_accounts: accounts1.clone(),
            })
            .await;
        assert!(res.is_ok());

        let res = store
            .add_action(StoreActions::AddOrganization {
                organization_id: org_id2,
                platform_accounts: accounts2.clone(),
            })
            .await;
        assert!(res.is_ok());

        let res = store.close().await;
        assert!(res.is_ok());

        let data = store.read_copy.read().await;
        let ids = &data.organization_ids;
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&org_id1));
        assert!(ids.contains(&org_id2));

        let org1 = store.get_random_organization().await;
        assert!(org1.id == org_id1 || org1.id == org_id2);
    }

    #[tokio::test]
    async fn test_add_existing_organization() {
        let store = Arc::new(Store::new());
        let org_id = 1;
        let accounts1 = vec![1, 2];
        let accounts2 = vec![3, 4];

        // Spawn the run task
        store.run().await;

        let res = store
            .add_action(StoreActions::AddOrganization {
                organization_id: org_id,
                platform_accounts: accounts1.clone(),
            })
            .await;
        assert!(res.is_ok());

        let res = store
            .add_action(StoreActions::AddOrganization {
                organization_id: org_id,
                platform_accounts: accounts2.clone(),
            })
            .await;
        assert!(res.is_ok());

        let res = store.close().await;
        assert!(res.is_ok());

        let org = store.get_random_organization().await;
        assert_eq!(org.platform_accounts, accounts2);
    }

    #[tokio::test]
    async fn test_larger_throughput() {
        let pre_tries = 100;
        let tries = 100000;

        let store = Arc::new(Store::new()); // Wrap in Arc

        // Spawn the run task
        store.run().await;

        let mut rng = rand::rng(); // Use thread_rng for better performance
                                   //
        async fn read_action(store: Arc<Store>) {
            let res_a = store.get_random_platform_account().await;
            let res_b = store.get_random_organization().await;

            assert_ne!(res_a.0, 0);
            assert_ne!(res_a.1, 0);
            assert_ne!(res_b.id, 0);
            assert_ne!(res_b.platform_accounts.len(), 0);
        }

        async fn write_action(
            rng: &mut ThreadRng,
            store: Arc<Store>,
        ) -> Result<(), SendError<Option<StoreActions>>> {
            let org_id: u64 = rng.random_range(0..u64::MAX);
            let platform_accounts_1: u64 = rng.random_range(0..u64::MAX);
            let platform_accounts_2: u64 = rng.random_range(0..u64::MAX);
            let platform_accounts_3: u64 = rng.random_range(0..u64::MAX);
            let platform_accounts_4: u64 = rng.random_range(0..u64::MAX);
            let platform_accounts_5: u64 = rng.random_range(0..u64::MAX);
            store
                .add_action(StoreActions::AddOrganization {
                    organization_id: org_id,
                    platform_accounts: vec![
                        platform_accounts_1,
                        platform_accounts_2,
                        platform_accounts_3,
                        platform_accounts_4,
                    ],
                })
                .await?;

            store
                .add_action(StoreActions::AddPlatformAccount {
                    organization_id: org_id,
                    platform_account_id: platform_accounts_5,
                })
                .await?;

            store
                .add_action(StoreActions::DeletePlatformAccount {
                    organization_id: org_id,
                    platform_account_id: platform_accounts_1,
                })
                .await?;

            Ok(())
        }

        for _ in 0..pre_tries {
            let res = write_action(&mut rng, store.clone()).await;
            assert!(res.is_ok());
        }

        sleep(Duration::from_millis(10)).await;
        for i in 0..tries {
            if i % 2 == 0 {
                let res = write_action(&mut rng, store.clone()).await;
                assert!(res.is_ok());
            } else {
                read_action(store.clone()).await;
            }
        }

        let res = store.close().await;

        assert!(res.is_ok());

        // Give the store some time to process the action

        let data = store.read_copy.read().await;
        let handled_count = store
            .handled_count
            .load(std::sync::atomic::Ordering::Relaxed);
        let ids = &data.organization_ids;

        let expected_handled_count = ((tries / 2) * 3) + (pre_tries * 3) + 1;
        let expected_org_count = (tries / 2) + pre_tries;
        assert_eq!(handled_count, expected_handled_count);
        assert_eq!(ids.len(), expected_org_count as usize);
    }
}
