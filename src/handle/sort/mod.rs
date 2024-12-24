pub use sort::sort_clients;
pub use update::update_clients;

#[allow(clippy::module_inception)]
mod sort;
#[cfg(test)]
mod tests;
mod update;
