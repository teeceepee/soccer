use std::collections::HashMap;
use std::hash::Hash;
use std::net::IpAddr;
use tokio::sync::oneshot;

enum CacheItem {
    Pending {
        responders: Vec<oneshot::Sender<Option<IpAddr>>>,
    },
    #[allow(dead_code)]
    Resolved {
        result: Option<IpAddr>,
    },
}

impl CacheItem {
    fn new_pending(responder: oneshot::Sender<Option<IpAddr>>) -> Self {
        Self::Pending {
            responders: vec![responder],
        }
    }
}

pub struct ResultCache<K>
    where K: Clone + Eq + Hash
{
    h_map: HashMap<K, CacheItem>,
}

impl<K> ResultCache<K>
    where K: Clone + Eq + Hash
{

    pub fn new() -> Self {
        Self {
            h_map: HashMap::new(),
        }
    }

    pub fn subscribe<F>(&mut self, key: &K, responder: oneshot::Sender<Option<IpAddr>>, f: F) -> ()
    where F: FnOnce()
    {
        match self.h_map.get_mut(key) {
            None => {
                let new_item = CacheItem::new_pending(responder);
                self.h_map.insert(key.clone(), new_item);
                f();
            }
            Some(item) => {
                match item {
                    CacheItem::Pending { responders } => {
                        responders.push(responder);
                    }
                    CacheItem::Resolved { .. } => {
                        todo!("asdf")
                    }
                }
            }
        }
    }

    pub fn notify(&mut self, key: &K, result: Option<IpAddr>) -> () {
        if let Some(item) = self.h_map.remove(key) {
            match item {
                CacheItem::Pending { responders } => {
                    for responder in responders {
                        let _ = responder.send(result);
                    }

                    // // Mark the item as resolved
                    // self.h_map.insert(key.clone(), CacheItem::Resolved { result });
                }
                CacheItem::Resolved { .. } => {
                    todo!()
                }
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr};
    use super::*;

    #[test]
    fn test_cache_item() {
        let (sender, _receiver) = oneshot::channel::<Option<IpAddr>>();
        let cache_item = CacheItem::new_pending(sender);

        let CacheItem::Pending { responders } = cache_item else {
            panic!();
        };
        assert_eq!(1, responders.len());
    }

    #[tokio::test]
    async fn test_dns_cache() {
        let mut c: ResultCache<String> = ResultCache { h_map: HashMap::new() };

        let (sender1, receiver1) = oneshot::channel::<Option<IpAddr>>();
        let (sender2, receiver2) = oneshot::channel::<Option<IpAddr>>();
        c.subscribe(&String::from("a"), sender1, || {});
        c.subscribe(&String::from("a"), sender2, || {});

        tokio::spawn(async move {
            let result = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
            c.notify(&"a".into(), Some(result));
        });

        let expected = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));

        let ret1 = receiver1.await.unwrap().unwrap();
        assert_eq!(expected, ret1);
        let ret2 = receiver2.await.unwrap().unwrap();
        assert_eq!(expected, ret2);
    }
}
