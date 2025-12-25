use crate::db::comments_repository::CommentsRepository;
use crate::db::links_repository::LinksRepository;

pub mod comments_repository;
pub mod links_repository;
pub mod postgresql;

pub trait CombinedRepository: CommentsRepository + LinksRepository {}
// TODO: explain this
impl<T: CommentsRepository + LinksRepository> CombinedRepository for T {}
