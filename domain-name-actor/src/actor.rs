use std::net::{IpAddr};
use tokio::sync::{mpsc, oneshot};
use domain_name_query_types::NameQuery;
use crate::resolve;
use crate::result_cache::ResultCache;

type QueryResult = Option<IpAddr>;

enum ActorMessage {
    Query {
        name_query: NameQuery,
        responder: oneshot::Sender<QueryResult>,
    },
    Notify {
        name_query: NameQuery,
        result: QueryResult,
    },
}

struct Actor {
    receiver: mpsc::Receiver<ActorMessage>,
    self_handle: ActorHandle,

    result_cache: ResultCache<NameQuery>,
    counter: u64,
}

impl Actor {
    pub fn new(receiver: mpsc::Receiver<ActorMessage>, handle: ActorHandle) -> Self {
        Self {
            receiver,
            self_handle: handle,

            result_cache: ResultCache::new(),
            counter: 0,
        }
    }

    pub async fn handle_message(&mut self, msg: ActorMessage) -> () {
        match msg {
            ActorMessage::Query { name_query, responder } => {
                self.counter += 1;
                let nq = name_query.clone();

                let self_handle = self.self_handle.clone();
                let f = || {
                    tokio::spawn(async move {
                        let ret = match resolve::resolve_domain(name_query.name.as_str()).await {
                            Ok(r) => { Some(r) }
                            Err(_) => { None }
                        };

                        self_handle.notify(name_query, ret).await;
                    });
                };

                self.result_cache.subscribe(&nq, responder, f);
            }
            ActorMessage::Notify { name_query, result } => {
                self.result_cache.notify(&name_query, result);
            }
        }
    }
}

async fn run_as_actor(mut actor: Actor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}

#[derive(Clone)]
pub struct ActorHandle {
    sender: mpsc::Sender<ActorMessage>,
}

impl ActorHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(10);
        let handle = Self { sender };
        let actor = Actor::new(receiver, handle.clone());
        tokio::spawn(run_as_actor(actor));

        handle
    }

    // call
    pub async fn query(&self, name_query: NameQuery) -> QueryResult {
        tracing::debug!("DNS query, {:?}", name_query);
        let (sender, receiver) = oneshot::channel();
        let msg = ActorMessage::Query {
            name_query,
            responder: sender,
        };

        let _ = self.sender.send(msg).await;
        let ret = receiver.await.unwrap();
        ret
    }

    // cast
    pub async fn notify(&self, name_query: NameQuery, result: QueryResult) {
        let msg = ActorMessage::Notify {
            name_query,
            result,
        };

        let _ = self.sender.send(msg).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_name_query_types::NameQuery;

    #[tokio::test]
    async fn test_foo() {
        let handle = ActorHandle::new();


        let tasks = (0..5).map(|i| {
            let handle = handle.clone();

            let name = if i % 2 == 0 {
                String::from("z.cn")
            } else {
                String::from("baidu.com")
            };
            let name_query = NameQuery::a_record(name.as_str());

            tokio::spawn(async move {
                let ret = handle.query(name_query).await;
                assert!(ret.is_some());
            })
        }).collect::<Vec<_>>();

        for t in tasks {
            let _ = t.await;
        }
    }
}
