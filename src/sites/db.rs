use crudcrate::{ApiError, CRUDResource, EntityToModels};
use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, DbBackend, Statement};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, EntityToModels)]
#[sea_orm(table_name = "sites")]
#[crudcrate(
    generate_router,
    api_struct = "Site",
    name_singular = "site",
    name_plural = "sites",
    description = "Collection sites with coordinates and associated site replicates",
    read::one::body = get_one_site_with_counts,
    join(name = "replicates", result = "Vec<crate::sites::replicates::db::SiteReplicate>", one, depth = 0),
    no_eq
)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[crudcrate(primary_key, filterable, exclude(update, create), on_create = Uuid::new_v4())]
    pub id: Uuid,

    #[sea_orm(unique)]
    #[crudcrate(sortable, filterable, fulltext)]
    pub name: String,
    #[crudcrate(sortable, filterable)]
    pub latitude_4326: f64,
    #[crudcrate(sortable, filterable)]
    pub longitude_4326: f64,
    #[crudcrate(sortable, filterable)]
    pub elevation_metres: f64,
    #[crudcrate(sortable, filterable)]
    pub area_id: Option<Uuid>,
    #[crudcrate(filterable, exclude(scoped))]
    pub is_private: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::sites::replicates::db::Entity")]
    SiteReplicates,
    #[sea_orm(has_one = "crate::areas::db::Entity")]
    Area,
}

impl Related<crate::sites::replicates::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SiteReplicates.def()
    }
}

impl Related<crate::areas::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Area.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub(super) async fn get_one_site_with_counts(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Site, ApiError> {
    use crate::sites::replicates::db::{Entity as ReplicateEntity, SiteReplicate};

    let model = Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::not_found("site", Some(id.to_string())))?;

    let replicate_models = model
        .find_related(ReplicateEntity)
        .all(db)
        .await?;

    let replicate_ids: Vec<Uuid> = replicate_models.iter().map(|r| r.id).collect();

    let counts = get_replicate_counts(db, &replicate_ids).await?;

    let replicates: Vec<SiteReplicate> = replicate_models
        .into_iter()
        .map(|m| {
            let (sc, ic, dc) = counts.get(&m.id).copied().unwrap_or((0, 0, 0));
            let mut rep = SiteReplicate::from(m);
            rep.sample_count = Some(sc);
            rep.isolate_count = Some(ic);
            rep.dna_count = Some(dc);
            rep
        })
        .collect();

    let mut site = Site::from(model);
    site.replicates = replicates;
    Ok(site)
}

async fn get_replicate_counts(
    db: &DatabaseConnection,
    replicate_ids: &[Uuid],
) -> Result<HashMap<Uuid, (i64, i64, i64)>, ApiError> {
    let mut result = HashMap::new();

    if replicate_ids.is_empty() {
        return Ok(result);
    }

    let raw_sql = r#"
        SELECT sr.id,
            COUNT(DISTINCT s.id) AS sample_count,
            COUNT(DISTINCT i.id) AS isolate_count,
            COUNT(DISTINCT d.id) AS dna_count
        FROM site_replicates sr
        LEFT JOIN samples s ON s.site_replicate_id = sr.id
        LEFT JOIN isolates i ON i.site_replicate_id = sr.id
        LEFT JOIN dna d ON d.site_replicate_id = sr.id
        WHERE sr.id = ANY($1)
        GROUP BY sr.id
    "#;

    let rows = db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            raw_sql,
            vec![replicate_ids.to_vec().into()],
        ))
        .await?;

    for row in rows {
        let id: Uuid = row.try_get("", "id")?;
        let sample_count: i64 = row.try_get("", "sample_count")?;
        let isolate_count: i64 = row.try_get("", "isolate_count")?;
        let dna_count: i64 = row.try_get("", "dna_count")?;
        result.insert(id, (sample_count, isolate_count, dna_count));
    }

    Ok(result)
}
