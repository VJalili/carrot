//! Contains structs and functions for doing operations on reports.
//!
//! A report, in this context, represents a template for a report file to be generated from a run.
//! The actual report entity only holds notebook, and the actual description of what will be
//! contained within the report is defined within the sections mapped to it. Represented in the
//! database by the REPORT table.

use crate::custom_sql_types::{ReportStatusEnum, REPORT_FAILURE_STATUSES};
use crate::schema::report;
use crate::schema::report::dsl::*;
use crate::schema::run_report;
use crate::util;
use chrono::NaiveDateTime;
use core::fmt;
use diesel::dsl::all;
use diesel::prelude::*;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Mapping to a report as it exists in the REPORT table in the database.
///
/// An instance of this struct will be returned by any queries for reports.
#[derive(Queryable, Serialize, Deserialize, PartialEq, Debug)]
pub struct ReportData {
    pub report_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub notebook: Value,
    pub config: Option<Value>,
    pub created_at: NaiveDateTime,
    pub created_by: Option<String>,
}

/// Represents all possible parameters for a query of the REPORT table
///
/// All values are optional, so any combination can be used during a query.  Limit and offset are
/// used for pagination.  Sort expects a comma-separated list of sort keys, optionally enclosed
/// with either asc() or desc().  For example: asc(name),desc(description),report_id
#[derive(Deserialize, Serialize)]
pub struct ReportQuery {
    pub report_id: Option<Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub notebook: Option<Value>,
    pub config: Option<Value>,
    pub created_before: Option<NaiveDateTime>,
    pub created_after: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// A new report to be inserted into the DB
///
/// name is a required field, but description, config, and created_by are not, so can be filled with
/// `None`. report_id and created_at are populated automatically by the DB
#[derive(Deserialize, Insertable, Serialize)]
#[table_name = "report"]
pub struct NewReport {
    pub name: String,
    pub description: Option<String>,
    pub notebook: Value,
    pub config: Option<Value>,
    pub created_by: Option<String>,
}

/// Represents fields to change when updating a report
///
/// Only name and description can be modified if there is a non-failed run report
/// associated with the specified report.  If there is not, notebook and config can also be modified
#[derive(Deserialize, Serialize, AsChangeset, Debug)]
#[table_name = "report"]
pub struct ReportChangeset {
    pub name: Option<String>,
    pub description: Option<String>,
    pub notebook: Option<Value>,
    pub config: Option<Value>,
}

/// Represents an error generated by an attempt at updating a row in the SECTION table
///
/// Updates can fail either because of a diesel error or because some of the parameters to be
/// updated are not allowed to be updated
#[derive(Debug)]
pub enum UpdateError {
    DB(diesel::result::Error),
    Prohibited(String),
}

impl std::error::Error for UpdateError {}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpdateError::DB(e) => write!(f, "UpdateError DB {}", e),
            UpdateError::Prohibited(e) => write!(f, "UpdateError Prohibited {}", e),
        }
    }
}

impl From<diesel::result::Error> for UpdateError {
    fn from(e: diesel::result::Error) -> UpdateError {
        UpdateError::DB(e)
    }
}

impl ReportData {
    /// Queries the DB for a report with the specified id
    ///
    /// Queries the DB using `conn` to retrieve the first row with a report_id value of `id`
    /// Returns a result containing either the retrieved report as a ReportData instance
    /// or an error if the query fails for some reason or if no report is found matching the
    /// criteria
    pub fn find_by_id(conn: &PgConnection, id: Uuid) -> Result<Self, diesel::result::Error> {
        report.filter(report_id.eq(id)).first::<Self>(conn)
    }

    /// Queries the DB for reports matching the specified query criteria
    ///
    /// Queries the DB using `conn` to retrieve reports matching the criteria in `params`
    /// Returns a result containing either a vector of the retrieved reports as ReportData
    /// instances or an error if the query fails for some reason
    pub fn find(
        conn: &PgConnection,
        params: ReportQuery,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        // Put the query into a box (pointer) so it can be built dynamically
        let mut query = report.into_boxed();

        // Add filters for each of the params if they have values
        if let Some(param) = params.report_id {
            query = query.filter(report_id.eq(param));
        }
        if let Some(param) = params.name {
            query = query.filter(name.eq(param));
        }
        if let Some(param) = params.description {
            query = query.filter(description.eq(param));
        }
        if let Some(param) = params.notebook {
            query = query.filter(notebook.eq(param));
        }
        if let Some(param) = params.config {
            query = query.filter(config.eq(param));
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

        // If there is a sort param, parse it and add to the order by clause accordingly
        if let Some(sort) = params.sort {
            let sort = util::sort_string::parse_sort_string(&sort);
            for sort_clause in sort {
                match &*sort_clause.key {
                    "report_id" => {
                        if sort_clause.ascending {
                            query = query.then_order_by(report_id.asc());
                        } else {
                            query = query.then_order_by(report_id.desc());
                        }
                    }
                    "name" => {
                        if sort_clause.ascending {
                            query = query.then_order_by(name.asc());
                        } else {
                            query = query.then_order_by(name.desc());
                        }
                    }
                    "description" => {
                        if sort_clause.ascending {
                            query = query.then_order_by(description.asc());
                        } else {
                            query = query.then_order_by(description.desc());
                        }
                    }
                    "notebook" => {
                        if sort_clause.ascending {
                            query = query.then_order_by(notebook.asc());
                        } else {
                            query = query.then_order_by(notebook.desc());
                        }
                    }
                    "config" => {
                        if sort_clause.ascending {
                            query = query.then_order_by(config.asc());
                        } else {
                            query = query.then_order_by(config.desc());
                        }
                    }
                    "created_at" => {
                        if sort_clause.ascending {
                            query = query.then_order_by(created_at.asc());
                        } else {
                            query = query.then_order_by(created_at.desc());
                        }
                    }
                    "created_by" => {
                        if sort_clause.ascending {
                            query = query.then_order_by(created_by.asc());
                        } else {
                            query = query.then_order_by(created_by.desc());
                        }
                    }
                    // Don't add to the order by clause if the sort key isn't recognized
                    &_ => {}
                }
            }
        }

        if let Some(param) = params.limit {
            query = query.limit(param);
        }
        if let Some(param) = params.offset {
            query = query.offset(param);
        }

        // Perform the query
        query.load::<Self>(conn)
    }

    /// Inserts a new report into the DB
    ///
    /// Creates a new report row in the DB using `conn` with the values specified in `params`
    /// Returns a result containing either the new report that was created or an error if the
    /// insert fails for some reason
    pub fn create(conn: &PgConnection, params: NewReport) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(report).values(&params).get_result(conn)
    }

    /// Updates a specified report in the DB
    ///
    /// Updates the report row in the DB using `conn` specified by `id` with the values in
    /// `params`.  Fails if trying to update notebook and there are nonfailed run_reports associated
    /// with this report
    /// Returns a result containing either the newly updated report or an error if the update
    /// fails for some reason
    pub fn update(
        conn: &PgConnection,
        id: Uuid,
        params: ReportChangeset,
    ) -> Result<Self, UpdateError> {
        // If trying to update the notebook, verify that no non-failed run_reports exist for this
        // report
        if matches!(params.notebook, Some(_)) {
            match Self::has_nonfailed_run_reports(conn, id) {
                // If there is a nonfailed run_report, return an error
                Ok(true) => {
                    let err = UpdateError::Prohibited(String::from("Attempted to update notebook when a non-failed run_report exists for this template.  Doing so is prohibited"));
                    error!("Failed to update due to error: {}", err);
                    return Err(err);
                }
                // If there are no nonfailed run_reports, don't stop execution
                Ok(false) => {}
                // If checking for run_reports failed for some reason, return the error
                Err(e) => {
                    error!("Failed to update due to error: {}", e);
                    return Err(UpdateError::DB(e));
                }
            }
        }
        Ok(diesel::update(report.filter(report_id.eq(id)))
            .set(params)
            .get_result(conn)?)
    }

    /// Deletes a specific report in the DB
    ///
    /// Deletes the report row in the DB using `conn` specified by `id`
    /// Returns a result containing either the number of rows deleted or an error if the delete
    /// fails for some reason
    pub fn delete(conn: &PgConnection, id: Uuid) -> Result<usize, diesel::result::Error> {
        diesel::delete(report.filter(report_id.eq(id))).execute(conn)
    }

    /// Checks whether the specified report has nonfailed run_reports associated with it
    ///
    /// Returns either a boolean indicating whether there are run_reports in the database that are
    /// children of the report specified by `id` that have non-failure statuses, or a diesel error
    /// if one is encountered
    pub fn has_nonfailed_run_reports(
        conn: &PgConnection,
        id: Uuid,
    ) -> Result<bool, diesel::result::Error> {
        // Query the run_reports table for non failed run reports
        let non_failed_run_reports_count = run_report::dsl::run_report
            .filter(run_report::dsl::report_id.eq(id))
            .filter(
                run_report::dsl::status.ne(all(REPORT_FAILURE_STATUSES
                    .iter()
                    .cloned()
                    .collect::<Vec<ReportStatusEnum>>())),
            )
            .select(run_report::dsl::run_id)
            .first::<Uuid>(conn);

        match non_failed_run_reports_count {
            // If we got a result, there is a nonfailed run_report, so return true
            Ok(_) => Ok(true),
            // If we got not found, then there are no nonfailed run_reports, so return false
            Err(diesel::result::Error::NotFound) => Ok(false),
            // Otherwise, return the error
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::custom_sql_types::RunStatusEnum;
    use crate::models::pipeline::{NewPipeline, PipelineData};
    use crate::models::run::{NewRun, RunData};
    use crate::models::run_report::{NewRunReport, RunReportData};
    use crate::models::template::{NewTemplate, TemplateData};
    use crate::models::test::{NewTest, TestData};
    use crate::unit_test_util::*;
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    fn insert_test_run(conn: &PgConnection) -> RunData {
        let new_pipeline = NewPipeline {
            name: String::from("Kevin's Pipeline 2"),
            description: Some(String::from("Kevin made this pipeline for testing 2")),
            created_by: Some(String::from("Kevin2@example.com")),
        };

        let pipeline =
            PipelineData::create(conn, new_pipeline).expect("Failed inserting test pipeline");

        let new_template = NewTemplate {
            name: String::from("Kevin's Template2"),
            pipeline_id: pipeline.pipeline_id,
            description: Some(String::from("Kevin made this template for testing2")),
            test_wdl: String::from("testtest"),
            eval_wdl: String::from("evaltest"),
            created_by: Some(String::from("Kevin2@example.com")),
        };

        let template =
            TemplateData::create(conn, new_template).expect("Failed inserting test template");

        let new_test = NewTest {
            name: String::from("Kevin's Test"),
            template_id: template.template_id,
            description: Some(String::from("Kevin made this test for testing")),
            test_input_defaults: Some(serde_json::from_str("{\"test\":\"test\"}").unwrap()),
            test_option_defaults: None,
            eval_input_defaults: Some(serde_json::from_str("{\"eval\":\"test\"}").unwrap()),
            eval_option_defaults: None,
            created_by: Some(String::from("Kevin@example.com")),
        };

        let test = TestData::create(conn, new_test).expect("Failed inserting test test");

        let new_run = NewRun {
            test_id: test.test_id,
            name: String::from("Kevin's test run"),
            status: RunStatusEnum::Succeeded,
            test_input: serde_json::from_str("{\"test\":\"1\"}").unwrap(),
            test_options: None,
            eval_input: serde_json::from_str("{}").unwrap(),
            eval_options: None,
            test_cromwell_job_id: Some(String::from("123456789")),
            eval_cromwell_job_id: Some(String::from("12345678902")),
            created_by: Some(String::from("Kevin@example.com")),
            finished_at: Some(Utc::now().naive_utc()),
        };

        RunData::create(&conn, new_run).expect("Failed to insert run")
    }

    fn insert_test_report(conn: &PgConnection) -> ReportData {
        let new_report = NewReport {
            name: String::from("Kevin's Report"),
            description: Some(String::from("Kevin made this report for testing")),
            notebook: json!({"cells":[{"test1":"test"}]}),
            config: Some(json!({"memory": "32 GiB"})),
            created_by: Some(String::from("Kevin@example.com")),
        };

        ReportData::create(conn, new_report).expect("Failed inserting test report")
    }

    fn insert_test_reports(conn: &PgConnection) -> Vec<ReportData> {
        let mut reports = Vec::new();

        let new_report = NewReport {
            name: String::from("Name1"),
            description: Some(String::from("Description4")),
            notebook: json!({"cells":[{"test1":"test"}]}),
            config: Some(json!({"cpu": "4"})),
            created_by: Some(String::from("Test@example.com")),
        };

        reports.push(ReportData::create(conn, new_report).expect("Failed inserting test report"));

        let new_report = NewReport {
            name: String::from("Name2"),
            description: Some(String::from("Description3")),
            notebook: json!({"cells":[{"test2":"test"}]}),
            config: None,
            created_by: Some(String::from("Test@example.com")),
        };

        reports.push(ReportData::create(conn, new_report).expect("Failed inserting test report"));

        let new_report = NewReport {
            name: String::from("Name4"),
            description: Some(String::from("Description3")),
            notebook: json!({"cells":[{"test3":"test"}]}),
            config: Some(json!({"preemptible": "1"})),
            created_by: Some(String::from("Test@example.com")),
        };

        reports.push(ReportData::create(conn, new_report).expect("Failed inserting test report"));

        reports
    }

    fn insert_test_run_report_failed(conn: &PgConnection) -> RunReportData {
        let run = insert_test_run(conn);

        let new_report = NewReport {
            name: String::from("Kevin's Report"),
            description: Some(String::from("Kevin made this report for testing")),
            notebook: json!({"notebook":[{"test3":"test"}]}),
            config: Some(json!({"cpu": "4"})),
            created_by: Some(String::from("Kevin@example.com")),
        };

        let new_report =
            ReportData::create(conn, new_report).expect("Failed inserting test report");

        let new_run_report = NewRunReport {
            run_id: run.run_id,
            report_id: new_report.report_id,
            status: ReportStatusEnum::Failed,
            cromwell_job_id: Some(String::from("testtesttesttest")),
            results: None,
            created_by: Some(String::from("Kevin@example.com")),
            finished_at: Some(Utc::now().naive_utc()),
        };

        RunReportData::create(conn, new_run_report).expect("Failed inserting test run_report")
    }

    fn insert_test_run_report_non_failed(conn: &PgConnection) -> RunReportData {
        let run = insert_test_run(conn);

        let new_report = NewReport {
            name: String::from("Kevin's Report"),
            description: Some(String::from("Kevin made this report for testing")),
            notebook: json!({"notebook":[{"test2":"test"}]}),
            config: Some(json!({"maxRetries": "4"})),
            created_by: Some(String::from("Kevin@example.com")),
        };

        let new_report =
            ReportData::create(conn, new_report).expect("Failed inserting test report");

        let new_run_report = NewRunReport {
            run_id: run.run_id,
            report_id: new_report.report_id,
            status: ReportStatusEnum::Running,
            cromwell_job_id: Some(String::from("testtesttesttest")),
            results: None,
            created_by: Some(String::from("Kevin@example.com")),
            finished_at: None,
        };

        RunReportData::create(conn, new_run_report).expect("Failed inserting test run_report")
    }

    #[test]
    fn find_by_id_exists() {
        let conn = get_test_db_connection();

        let test_report = insert_test_report(&conn);

        let found_report = ReportData::find_by_id(&conn, test_report.report_id)
            .expect("Failed to retrieve test report by id.");

        assert_eq!(found_report, test_report);
    }

    #[test]
    fn find_by_id_not_exists() {
        let conn = get_test_db_connection();

        let nonexistent_report = ReportData::find_by_id(&conn, Uuid::new_v4());

        assert!(matches!(
            nonexistent_report,
            Err(diesel::result::Error::NotFound)
        ));
    }

    #[test]
    fn find_with_report_id() {
        let conn = get_test_db_connection();

        let test_reports = insert_test_reports(&conn);

        let test_query = ReportQuery {
            report_id: Some(test_reports[0].report_id),
            name: None,
            description: None,
            notebook: None,
            config: None,
            created_before: None,
            created_after: None,
            created_by: None,
            sort: None,
            limit: None,
            offset: None,
        };

        let found_reports = ReportData::find(&conn, test_query).expect("Failed to find reports");

        assert_eq!(found_reports.len(), 1);
        assert_eq!(found_reports[0], test_reports[0]);
    }

    #[test]
    fn find_with_name() {
        let conn = get_test_db_connection();

        let test_reports = insert_test_reports(&conn);

        let test_query = ReportQuery {
            report_id: None,
            name: Some(test_reports[0].name.clone()),
            description: None,
            notebook: None,
            config: None,
            created_before: None,
            created_after: None,
            created_by: None,
            sort: None,
            limit: None,
            offset: None,
        };

        let found_reports = ReportData::find(&conn, test_query).expect("Failed to find reports");

        assert_eq!(found_reports.len(), 1);
        assert_eq!(found_reports[0], test_reports[0]);
    }

    #[test]
    fn find_with_description() {
        let conn = get_test_db_connection();

        let test_reports = insert_test_reports(&conn);

        let test_query = ReportQuery {
            report_id: None,
            name: None,
            description: Some(test_reports[0].description.clone().unwrap()),
            notebook: None,
            config: None,
            created_before: None,
            created_after: None,
            created_by: None,
            sort: None,
            limit: None,
            offset: None,
        };

        let found_reports = ReportData::find(&conn, test_query).expect("Failed to find reports");

        assert_eq!(found_reports.len(), 1);
        assert_eq!(found_reports[0], test_reports[0]);
    }

    #[test]
    fn find_with_notebook() {
        let conn = get_test_db_connection();

        let test_reports = insert_test_reports(&conn);

        let test_query = ReportQuery {
            report_id: None,
            name: None,
            description: None,
            notebook: Some(json!({"cells":[{"test1":"test"}]})),
            config: None,
            created_before: None,
            created_after: None,
            created_by: None,
            sort: None,
            limit: None,
            offset: None,
        };

        let found_reports = ReportData::find(&conn, test_query).expect("Failed to find reports");

        assert_eq!(found_reports.len(), 1);
        assert_eq!(found_reports[0], test_reports[0]);
    }

    #[test]
    fn find_with_config() {
        let conn = get_test_db_connection();

        let test_reports = insert_test_reports(&conn);

        let test_query = ReportQuery {
            report_id: None,
            name: None,
            description: None,
            notebook: None,
            config: Some(json!({"cpu": "4"})),
            created_before: None,
            created_after: None,
            created_by: None,
            sort: None,
            limit: None,
            offset: None,
        };

        let found_reports = ReportData::find(&conn, test_query).expect("Failed to find reports");

        assert_eq!(found_reports.len(), 1);
        assert_eq!(found_reports[0], test_reports[0]);
    }

    #[test]
    fn find_with_sort_and_limit_and_offset() {
        let conn = get_test_db_connection();

        let test_reports = insert_test_reports(&conn);

        let test_query = ReportQuery {
            report_id: None,
            name: None,
            description: None,
            notebook: None,
            config: None,
            created_before: None,
            created_after: None,
            created_by: Some(String::from("Test@example.com")),
            sort: Some(String::from("description,desc(name)")),
            limit: Some(2),
            offset: None,
        };

        let found_reports = ReportData::find(&conn, test_query).expect("Failed to find reports");

        assert_eq!(found_reports.len(), 2);
        assert_eq!(found_reports[0], test_reports[2]);
        assert_eq!(found_reports[1], test_reports[1]);

        let test_query = ReportQuery {
            report_id: None,
            name: None,
            description: None,
            notebook: None,
            config: None,
            created_before: None,
            created_after: None,
            created_by: Some(String::from("Test@example.com")),
            sort: Some(String::from("description,desc(name)")),
            limit: Some(2),
            offset: Some(2),
        };

        let found_reports = ReportData::find(&conn, test_query).expect("Failed to find reports");

        assert_eq!(found_reports.len(), 1);
        assert_eq!(found_reports[0], test_reports[0]);
    }

    #[test]
    fn find_with_created_before_and_created_after() {
        let conn = get_test_db_connection();

        insert_test_reports(&conn);

        let test_query = ReportQuery {
            report_id: None,
            name: None,
            description: None,
            notebook: None,
            config: None,
            created_before: None,
            created_after: Some("2099-01-01T00:00:00".parse::<NaiveDateTime>().unwrap()),
            created_by: Some(String::from("Test@example.com")),
            sort: None,
            limit: None,
            offset: None,
        };

        let found_reports = ReportData::find(&conn, test_query).expect("Failed to find reports");

        assert_eq!(found_reports.len(), 0);

        let test_query = ReportQuery {
            report_id: None,
            name: None,
            description: None,
            notebook: None,
            config: None,
            created_before: Some("2099-01-01T00:00:00".parse::<NaiveDateTime>().unwrap()),
            created_after: None,
            created_by: Some(String::from("Test@example.com")),
            sort: None,
            limit: None,
            offset: None,
        };

        let found_reports = ReportData::find(&conn, test_query).expect("Failed to find reports");

        assert_eq!(found_reports.len(), 3);
    }

    #[test]
    fn create_success() {
        let conn = get_test_db_connection();

        let test_report = insert_test_report(&conn);

        assert_eq!(test_report.name, "Kevin's Report");
        assert_eq!(
            test_report
                .description
                .expect("Created report missing description"),
            "Kevin made this report for testing"
        );
        assert_eq!(
            test_report
                .created_by
                .expect("Created report missing created_by"),
            "Kevin@example.com"
        );
    }

    #[test]
    fn create_failure_same_name() {
        let conn = get_test_db_connection();

        let test_report = insert_test_report(&conn);

        let copy_report = NewReport {
            name: test_report.name,
            description: test_report.description,
            config: None,
            notebook: json!({"notebook":[{"test1":"test"}]}),
            created_by: test_report.created_by,
        };

        let new_report = ReportData::create(&conn, copy_report);

        assert!(matches!(
            new_report,
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ),)
        ));
    }

    #[test]
    fn update_success() {
        let conn = get_test_db_connection();

        let test_report = insert_test_report(&conn);

        let changes = ReportChangeset {
            name: Some(String::from("TestTestTestTest")),
            description: Some(String::from("TESTTESTTESTTEST")),
            notebook: Some(json!({"notebook":[{"test1":"test"}]})),
            config: None,
        };

        let updated_report = ReportData::update(&conn, test_report.report_id, changes)
            .expect("Failed to update report");

        assert_eq!(updated_report.name, String::from("TestTestTestTest"));
        assert_eq!(
            updated_report.description.unwrap(),
            String::from("TESTTESTTESTTEST")
        );
    }

    #[test]
    fn update_failure_same_name() {
        let conn = get_test_db_connection();

        let test_reports = insert_test_reports(&conn);

        let changes = ReportChangeset {
            name: Some(test_reports[0].name.clone()),
            description: None,
            notebook: None,
            config: None,
        };

        let updated_report = ReportData::update(&conn, test_reports[1].report_id, changes);

        assert!(matches!(
            updated_report,
            Err(UpdateError::DB(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ),),)
        ));
    }

    #[test]
    fn delete_success() {
        let conn = get_test_db_connection();

        let test_report = insert_test_report(&conn);

        let delete_result = ReportData::delete(&conn, test_report.report_id).unwrap();

        assert_eq!(delete_result, 1);

        let deleted_report = ReportData::find_by_id(&conn, test_report.report_id);

        assert!(matches!(
            deleted_report,
            Err(diesel::result::Error::NotFound)
        ));
    }

    #[test]
    fn delete_failure_foreign_key() {
        let conn = get_test_db_connection();

        let test_run_report = insert_test_run_report_non_failed(&conn);

        let delete_result = ReportData::delete(&conn, test_run_report.report_id);

        assert!(matches!(
            delete_result,
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ),)
        ));
    }

    #[test]
    fn has_non_failed_run_reports_true() {
        let conn = get_test_db_connection();

        let test_run_report = insert_test_run_report_non_failed(&conn);

        let result =
            ReportData::has_nonfailed_run_reports(&conn, test_run_report.report_id).unwrap();

        assert!(result);
    }

    #[test]
    fn has_non_failed_run_reports_false() {
        let conn = get_test_db_connection();

        let test_run_report = insert_test_run_report_failed(&conn);

        let result =
            ReportData::has_nonfailed_run_reports(&conn, test_run_report.report_id).unwrap();

        assert!(!result);
    }

    #[test]
    fn has_non_failed_run_reports_false_no_runs() {
        let conn = get_test_db_connection();

        let test_report = insert_test_report(&conn);

        let result = ReportData::has_nonfailed_run_reports(&conn, test_report.report_id).unwrap();

        assert!(!result);
    }
}
