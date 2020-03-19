use crate::schema::test_framework::pipeline::dsl::*;
use crate::schema::test_framework::pipeline;
use crate::util;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{ Deserialize, Serialize };
use uuid::Uuid;

#[derive(Queryable, Serialize)]
pub struct Pipeline {
    pub pipeline_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub created_by: Option<String>,
}

#[derive(Deserialize)]
pub struct PipelineQuery {
    pub pipeline_id: Option<Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_before: Option<NaiveDateTime>,
    pub created_after: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Insertable)]
#[table_name="pipeline"]
pub struct NewPipeline {
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<String>,
}

impl Pipeline {

    pub fn find_by_id(conn: &PgConnection, id: Uuid) -> Result<Vec<Self>, diesel::result::Error> {
        pipeline.filter(pipeline_id.eq(id))
            .load::<Pipeline>(conn)
    }

    pub fn find(conn: &PgConnection, params: PipelineQuery) -> Result<Vec<Self>, diesel::result::Error> {

        let mut query = pipeline.into_boxed();

        if let Some(param) = params.pipeline_id {
            query = query.filter(pipeline_id.eq(param));
        }
        if let Some(param) = params.name {
            query = query.filter(name.eq(param));
        }
        if let Some(param) = params.description {
            query = query.filter(description.eq(param));
        }
        if let Some(param) = params.created_before {
            query = query.filter(created_at.lt(param));
        }
        if let Some(param) = params.created_after {
            query = query.filter(created_at.gt(param));
        }
        if let Some(param) = params.created_by {
            query = query.filter(created_by.eq(param));
        }

        if let Some(sort) = params.sort {
            let sort = util::parse_sort_string(sort);
            for sort_clause in sort {
                match &sort_clause.key[..] {
                    "pipeline_id" => {
                        if sort_clause.ascending {
                            query = query.then_order_by(pipeline_id.asc());
                        } else {
                            query = query.then_order_by(pipeline_id.desc());
                        }
                    },
                    "name" => {
                        if sort_clause.ascending {
                            query = query.then_order_by(name.asc());
                        } else {
                            query = query.then_order_by(name.desc());
                        }
                    },
                    "description" => {
                        if sort_clause.ascending {
                            query = query.then_order_by(description.asc());
                        } else {
                            query = query.then_order_by(description.desc());
                        }
                    },
                    "created_at" => {
                        if sort_clause.ascending {
                            query = query.then_order_by(created_at.asc());
                        } else {
                            query = query.then_order_by(created_at.desc());
                        }
                    },
                    "created_by" => {
                        if sort_clause.ascending {
                            query = query.then_order_by(created_by.asc());
                        } else {
                            query = query.then_order_by(created_by.desc());
                        }
                    },
                    &_ => {

                    }
                }
            }
        }

        if let Some(param) = params.limit {
            query = query.limit(param);
        }
        if let Some(param) = params.offset {
            query = query.offset(param);
        }

        query.load::<Pipeline>(conn)

    }

    pub fn create(conn: &PgConnection, params: NewPipeline) -> Result<Pipeline, diesel::result::Error> {
        diesel::insert_into(pipeline)
            .values(&params)
            .get_result(conn)
    }
}
