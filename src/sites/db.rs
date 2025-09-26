use crudcrate::{CRUDResource, EntityToModels};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, EntityToModels)]
#[sea_orm(table_name = "sites")]
#[crudcrate(
    generate_router,
    api_struct = "Site",
    name_singular = "site",
    name_plural = "sites",
    description = "Collection sites with coordinates and associated site replicates",
    // fn_get_all = get_all_sites_with_replicates,
    // fn_get_one = get_one_site_with_replicates,
    no_eq
)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[crudcrate(primary_key, exclude(update, create), on_create = Uuid::new_v4())]
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
    #[crudcrate(filterable)]
    pub is_private: bool,
    #[sea_orm(ignore)]
    #[crudcrate(non_db_attr, exclude(create, update), join(one, all))]
    pub replicates: Vec<crate::sites::replicates::db::SiteReplicate>,
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

// // Custom get_all function that includes site replicates
// pub(super) async fn get_all_sites_with_replicates(
//     db: &DatabaseConnection,
//     condition: &Condition,
//     order_column: Column,
//     order_direction: Order,
//     offset: u64,
//     limit: u64,
// ) -> Result<Vec<SiteList>, DbErr> {
//     let models = Entity::find()
//         .filter(condition.clone())
//         .order_by(order_column, order_direction)
//         .offset(offset)
//         .limit(limit)
//         .all(db)
//         .await?;

//     // Load replicates for all sites using the loader
//     let replicates = models
//         .load_many(crate::sites::replicates::db::Entity, db)
//         .await?;

//     let mut sites = Vec::new();
//     for (model, site_replicates) in models.into_iter().zip(replicates.into_iter()) {
//         let mut site: SiteList = model.into();
//         site.replicates = site_replicates.into_iter().map(|r| r.into()).collect();
//         sites.push(site);
//     }

//     Ok(sites)
// }

// // Custom get_one function that includes site replicates
// pub(super) async fn get_one_site_with_replicates(
//     db: &DatabaseConnection,
//     id: Uuid,
// ) -> Result<Site, DbErr> {
//     let model = Entity::find_by_id(id)
//         .one(db)
//         .await?
//         .ok_or(DbErr::RecordNotFound("Site not found".to_string()))?;

//     // Load replicates for this site
//     let replicates = model
//         .find_related(crate::sites::replicates::db::Entity)
//         .all(db)
//         .await?;

//     let mut site: Site = model.into();
//     site.replicates = replicates.into_iter().map(|r| r.into()).collect();

//     Ok(site)
// }
