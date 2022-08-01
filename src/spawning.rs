use crate::*;
use futures::Future;
use std::{sync::Arc, task::Poll};

/// Spawn a new `Actor` with a single `Process`. This will return a [Child] and
/// and [Address]. The `Process` is spawned with a single [Inbox].
///
/// This will immeadeately start the spawning. `await`-ing the [Spawn]-future will
/// wait until the [Channel] is fully initialized.
///
/// # Example
/// ```no_run
///# use tiny_actor::*;
///# #[tokio::main]
///# async fn main() {
/// let (child, address) =
///     spawn(Config::default(), |mut inbox: Inbox<u32>| async move {
///         loop {
///             let msg = inbox.recv().await;
///             println!("Received message: {msg:?}");
///         }
///     });
///# }
/// ```
pub fn spawn<M, E, Fun, Fut>(config: Config, fun: Fun) -> (Child<E, Channel<M>>, Address<M>)
where
    Fun: FnOnce(Inbox<M>) -> Fut + Send + 'static,
    Fut: Future<Output = E> + Send + 'static,
    E: Send + 'static,
    M: Send + 'static,
{
    let channel = Arc::new(Channel::<M>::new(1, 1, config.capacity));
    let address = Address::from_channel(channel.clone());
    let inbox = Inbox::from_channel(channel.clone());

    let handle = tokio::task::spawn(async move { fun(inbox).await });

    let child = Child::new(channel, handle, config.link);

    (child, address)
}

/// Same as [spawn], but returns a [ChildPool] instead of a [Child].
///
/// # Example
/// ```no_run
///# use tiny_actor::*;
///# #[tokio::main]
///# async fn main() {
/// let (child_pool, address) =
///     spawn_one(Config::default(), |mut inbox: Inbox<u32>| async move {
///         loop {
///             let msg = inbox.recv().await;
///             println!("Received message: {msg:?}");
///         }
///     });
///# }
/// ```
pub fn spawn_one<M, E, Fun, Fut>(config: Config, fun: Fun) -> (ChildPool<E, Channel<M>>, Address<M>)
where
    Fun: FnOnce(Inbox<M>) -> Fut + Send + 'static,
    Fut: Future<Output = E> + Send + 'static,
    E: Send + 'static,
    M: Send + 'static,
{
    let channel = Arc::new(Channel::<M>::new(1, 1, config.capacity));
    let address = Address::from_channel(channel.clone());
    let inbox = Inbox::from_channel(channel.clone());

    let handle = tokio::task::spawn(async move { fun(inbox).await });

    let child = ChildPool::new(channel, vec![handle], config.link);

    (child, address)
}



/// Spawn a new `Actor` with a multiple `Process`es. This will return a [ChildPool] and
/// and [Address]. The `Process`es are spawned with [Inbox]es.
///
/// The amount of `Process`es that are spawned is equal to the length of the iterator.
/// Every process get's access to a single item within the iterator as it's first argument.
///
/// # Example
/// ```no_run
///# use tiny_actor::*;
///# #[tokio::main]
///# async fn main() {
/// let (child_pool, address) =
///     spawn_many(0..5, Config::default(), |i, mut inbox: Inbox<u32>| async move {
///         loop {
///             let msg = inbox.recv().await;
///             println!("Received message on actor {i}: {msg:?}");
///         }
///     });
///# }
/// ```
pub fn spawn_many<M, E, I, Fun, Fut>(
    iter: impl IntoIterator<Item = I>,
    config: Config,
    fun: Fun,
) -> (ChildPool<E, Channel<M>>, Address<M>)
where
    Fun: FnOnce(I, Inbox<M>) -> Fut + Send + 'static + Clone,
    Fut: Future<Output = E> + Send + 'static,
    E: Send + 'static,
    M: Send + 'static,
    I: Send + 'static,
{
    let iter = iter.into_iter();
    let channel = Arc::new(Channel::<M>::new(1, 1, config.capacity));
    let address = Address::from_channel(channel.clone());

    let handles = iter
        .map(|i| {
            let fun = fun.clone();
            let inbox = Inbox::from_channel(address.channel().clone());
            tokio::task::spawn(async move { fun(i, inbox).await })
        })
        .collect::<Vec<_>>();

    address.channel().set_inbox_count(handles.len());

    let child = ChildPool::new(channel, handles, config.link);

    (child, address)
}
