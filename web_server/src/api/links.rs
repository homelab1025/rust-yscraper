use crate::api::app_state::AppState;
use crate::api::common::{ApiError, ApiErrorCode};
use crate::db::links_repository::LinksRepository;
use axum::Json;
use axum::extract::{FromRef, Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Clone)]
pub struct LinksAppState {
    pub repo: Arc<dyn LinksRepository>,
}

impl FromRef<AppState> for LinksAppState {
    fn from_ref(input: &AppState) -> Self {
        LinksAppState {
            repo: input.repo.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq)]
pub struct LinkDto {
    pub id: i64,
    pub url: String,
    pub date_added: String,
}

/// Retrieve all links with their item IDs and added date.
#[utoipa::path(
    get,
    path = "/links",
    responses(
        (status = 200, description = "List of all links", body = [LinkDto]),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn list_links(
    State(state): State<LinksAppState>,
) -> Result<Json<Vec<LinkDto>>, (StatusCode, Json<ApiError>)> {
    let links = state.repo.list_links().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: ApiErrorCode::DatabaseError,
                msg: format!("Database error: {}", e),
            }),
        )
    })?;

    let dtos = links
        .into_iter()
        .map(|row| LinkDto {
            id: row.id,
            url: row.url,
            date_added: row.date_added.to_rfc3339(),
        })
        .collect();

    Ok(Json(dtos))
}

#[utoipa::path(
    delete,
    path = "/links/{id}",
    responses(
        (status = 200, description = "Link deleted successfully"),
        (status = 404, description = "Link not found", body = ApiError)
    )
)]
pub async fn delete_link(
    State(state): State<LinksAppState>,
    Path(id): Path<i64>,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    match state.repo.delete_link(id).await {
        Ok(n) => {
            if n == 0 {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiError {
                        code: ApiErrorCode::NotFound,
                        msg: format!("Link with ID {} not found", id),
                    }),
                ))
            } else {
                Ok(())
            }
        }
        Err(e) => {
            log::error!("Failed to delete link: {}", e);

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: ApiErrorCode::DatabaseError,
                    msg: format!("Failed to delete link: {}", e),
                }),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::links_repository::DbUrlRow;
    use async_trait::async_trait;
    use chrono::Utc;
    use sqlx::Error;

    struct MockLinksRepository {
        links: Result<Vec<DbUrlRow>, String>,
        delete_result: Result<u64, String>,
    }

    #[async_trait]
    impl LinksRepository for MockLinksRepository {
        async fn list_links(&self) -> Result<Vec<DbUrlRow>, Error> {
            self.links.clone().map_err(Error::Protocol)
        }

        async fn delete_link(&self, _id: i64) -> Result<u64, sqlx::Error> {
            self.delete_result.clone().map_err(Error::Protocol)
        }
    }

    #[tokio::test]
    async fn test_list_links_success() {
        let time_url1 = Utc::now();
        let time_url2 = Utc::now();
        let rows = vec![
            DbUrlRow {
                id: 1,
                url: "https://example.com/1".to_string(),
                date_added: time_url1,
            },
            DbUrlRow {
                id: 2,
                url: "https://example.com/2".to_string(),
                date_added: time_url2,
            },
        ];

        let state = LinksAppState {
            repo: Arc::new(MockLinksRepository {
                links: Ok(rows.clone()),
                delete_result: Ok(0),
            }),
        };

        let result = list_links(State(state)).await;
        assert!(result.is_ok());

        let Json(links) = result.unwrap();
        assert_eq!(links.len(), 2);

        assert!(links.contains(&LinkDto {
            id: 1,
            url: "https://example.com/1".to_string(),
            date_added: time_url1.to_rfc3339(),
        }));

        assert!(links.contains(&LinkDto {
            id: 2,
            url: "https://example.com/2".to_string(),
            date_added: time_url2.to_rfc3339(),
        }));
    }

    #[tokio::test]
    async fn test_list_links_empty() {
        let state = LinksAppState {
            repo: Arc::new(MockLinksRepository {
                links: Ok(vec![]),
                delete_result: Ok(0),
            }),
        };

        let result = list_links(State(state)).await;
        assert!(result.is_ok());

        let Json(links) = result.unwrap();
        assert_eq!(links.len(), 0);
    }

    #[tokio::test]
    async fn test_list_links_db_error() {
        let state = LinksAppState {
            repo: Arc::new(MockLinksRepository {
                links: Err("DB error".to_string()),
                delete_result: Ok(0),
            }),
        };

        let result = list_links(State(state)).await;
        assert!(result.is_err());

        let (status, Json(err)) = result.unwrap_err();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, ApiErrorCode::DatabaseError);
    }

    #[tokio::test]
    async fn test_delete_link_success() {
        let state = LinksAppState {
            repo: Arc::new(MockLinksRepository {
                links: Ok(vec![]),
                delete_result: Ok(1),
            }),
        };

        let result = delete_link(State(state), Path(1)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_link_not_found() {
        let state = LinksAppState {
            repo: Arc::new(MockLinksRepository {
                links: Ok(vec![]),
                delete_result: Ok(0),
            }),
        };

        let result = delete_link(State(state), Path(1)).await;
        assert!(result.is_err());

        let (status, Json(err)) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(err.code, ApiErrorCode::NotFound);
        assert!(err.msg.contains("Link with ID 1 not found"));
    }

    #[tokio::test]
    async fn test_delete_link_db_error() {
        let state = LinksAppState {
            repo: Arc::new(MockLinksRepository {
                links: Ok(vec![]),
                delete_result: Err("DB error".to_string()),
            }),
        };

        let result = delete_link(State(state), Path(1)).await;
        assert!(result.is_err());

        let (status, Json(err)) = result.unwrap_err();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, ApiErrorCode::DatabaseError);
        assert!(err.msg.contains("Failed to delete link"));
    }
}
