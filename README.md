![CARROT](logo.png)
# CARROT

This repository contains the Cromwell Automated Runner for Regression and Automation Testing.  This is a tool for configuring, running, and comparing the results of tests run in the [Cromwell Workflow Engine](https://github.com/broadinstitute/cromwell).

## Table of Contents
* [Requirements](#requirements)
    * [Building and Running CARROT](#building_and_running)
    * [Dynamic Software Testing](#software_building)
    * [Email Notifications](#email_notifications)
    * [GitHub Integration](#github_integration)
    * [Reporting](#reporting)
* [Style](#style)
* [Using CARROT](#using_carrot)
* [License](#license)
* [Versioning](#versioning)

## <a name="requirements">Requirements</a>

### <a name="building_and_running">Building and Running CARROT</a>
* A Rust version >=1.51.0 is required to build CARROT
    * rustup, the installer for Rust, can be found on the Rust website, [here](https://www.rust-lang.org/tools/install).
    * rustup will install the Rust compiler (rustc) and the Rust package manager (Cargo).
* CARROT currently requires a PostgreSQL database with version >=12.2 for storing test information.
    * PostgreSQL can be downloaded from the PostgreSQL website, [here](https://www.postgresql.org/download/).
    * It is also a requirement that the PostgreSQL DB have the `uuid-ossp` extension for using UUIDs.
        * This extension can be installed by connecting to the database as a user with SUPERUSER privileges and running the following command:
        `create extension if not exists "uuid-ossp";`
* Certain configuration information must be specified in config variables before running.
    * These variables can be specified using a `.yml` file.  An example of a `.yml` configuration can be found within the `carrot.example.yml` file.
* CARROT uses the [Diesel](http://diesel.rs/) crate for interfacing with the database.  For certain dev and build tasks, the Diesel CLI is required.
    * Instructions for installing the Diesel CLI can be found [here](http://diesel.rs/guides/getting-started/).
    * Once the Diesel CLI is installed and the PostgreSQL database is running, the Diesel CLI migration tool can be used to create all of the required tables and types in the database with the command `diesel migration run`
    * Alternatively, these tables and types will all be created when running CARROT for the first time
* CARROT uses [womtool](https://cromwell.readthedocs.io/en/develop/WOMtool/) for WDL validation.  If running outside of a docker container created using the included Dockerfile, it will be necessary to include the womtool jar on the same machine and set the `womtool_location` config variable to its location, as shown in the `carrot.example.yml` file
* Once Rust is installed, the project can be built using the `cargo build` command in the project directory.
    * Building for release can be done using `cargo build --release`
* CARROT requires a [Cromwell](https://github.com/broadinstitute/cromwell) server to run tests
    * Setting up a Cromwell server can be accomplished by following the instructions [here](https://docs.google.com/document/d/1FlKe3XvjzE2-Yzi245THpC6X7D0opRufjh7Mt21bBhE/edit?usp=sharing)
* A Dockerfile is provided in the project root directory that can be used to run CARROT in a Docker container.
    * The image can be built by running `docker build .` from the project root.
* For development purposes, the `scripts/docker/docker-compose.yml` file can be used to run CARROT with a PostreSQL server and a Cromwell server in their own containers.  This can be done using `docker-compose build` followed by `docker-compose up` within that directory.
    * Running CARROT in this way uses the bare minimum features for CARROT and Cromwell, so software building, reporting, and GitHub integration are unavailable with the default configuration in that `docker-compose.yml` file.  Since we're using minimum features for Cromwell, it will also be impossible to access Cromwell job metadata between restarts of the container (although the actual data for jobs will be retained within the volume).
    * Accessing result files when running like this requires accessing the docker volume which contains the data for the cromwell instance.  This will be within a directory called `docker_cromwell-data` within the Docker volumes directory on your machine.
        * Note: if you are running Docker on Mac, it is necessary to connect to the Docker VM to access the volumes directory.  Recent versions of Docker have a bug preventing doing so via `screen`, so the easiest way to do it is to connect via a container using `docker run -it --privileged --pid=host debian nsenter -t 1 -m -u -n -i sh`
* To run unit tests in Docker, use `docker-compose build` followed by `docker-compose up --abort-on-container-exit --exit-code-from carrot-test` within the `/scripts/docker/test` directory.
    * Building and running tests this way seems to occasionally result in failures with the carrot-test container being killed because it exceeds the default docker memory allocation while compiling.  This can be resolved by increasing your allocated memory for docker in your docker settings.

### <a name="email_notifications">Email Notifications</a>
* CARROT supports the option of sending email notifications to subscribed users upon completion of a test run.  
    * Emails can be configured to be sent in the following ways:
        * Using the local machine's `sendmail` utility, or
        * Using an SMTP mail server (either running your own, or using an existing mail service like GMail).
    * Enabling this requires the use of a few configuration variables which are listed and explained in the `carrot.example.yml` file.

### <a name="software_building">Dynamic Software Testing</a>
* It is possible (and encouraged) to set up CARROT to allow automatic generation of docker images for testing specific software hosted in a git repository
* In order to allow this for private GitHub repos, it is necessary to set up private github access configuration as detailed in the `carrot.example.yml` file

### <a name="github_integration">GitHub Integration</a>
* CARROT supports triggering runs via GitHub PR comments, and receiving reply comments with run results.
* Enabling this functionality requires multiple steps:
    * Set up a [Google Cloud PubSub Topic](https://cloud.google.com/pubsub/docs/overview)
        * CARROT will use the created subscription to read messages to trigger runs from the topic
    * Create a GitHub account for CARROT to use to view and interact with GitHub
    * Add the [carrot-publish-github-action](https://github.com/broadinstitute/carrot-publish-github-action) to the GitHub Actions workflow for the repository you want to test
        * Instructions for doing so are included in the README for the action
    * Set up the `github` configuration as detailed in the `carrot.example.yml` file

### <a name="reporting">Reporting</a>
* An important functionality of CARROT is the generation of reports from test runs in the form of Jupyter Notebooks
* In order for this functionality to work properly, it is necessary to:
    * Set up the `reporting` config in the config yaml file
        * Create a Google Cloud bucket for storing report templates and use it as the value for the `report_location` variable
        * Build and push the report Dockerfile (`scripts/docker/reports/Dockerfile`) to a repository accessible by the Google Cloud service account associated with your Cromwell instance
            * Also set the `report_docker_location` variable to its location
            * Alternatively, you can build a docker image with Jupyter Notebook support and the libraries you need if the provided Dockerfile does not meet your needs

## <a name="style">Style</a>

When contributing to CARROT, you should do your best to adhere to the [Rust style guide](https://github.com/rust-dev-tools/fmt-rfcs/blob/master/guide/guide.md).

To make adhering to the style guide easier, there is a Rust automatic formatting tool called [rustfmt](https://github.com/rust-lang/rustfmt). This tool can be installed with cargo using the command `rustup component add rustfmt` and should be run using `cargo fmt` before making a pull request.

## <a name="using_carrot">Using CARROT</a>

Once you have a CARROT server running, please see the [User Guide](UserGuide.md) for instructions on using CARROT.

There is also an [example test repo](https://github.com/broadinstitute/carrot-example-test) available with instructions on how to create and run an example test on your CARROT server.

## <a name="license">License</a>

Licensed under Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE)) AND MIT License ([LICENSE-MIT](LICENSE-MIT))

### <a name="versioning">Versioning</a>

We use `bumpversion.sh` to maintain version numbers.
DO NOT MANUALLY EDIT ANY VERSION NUMBERS.

Our versions are specified by a 3 number semantic version system (https://semver.org/):

	major.minor.patch

To update the version do the following:

`./bumpversion.sh PART` where PART is one of:
- major
- minor
- patch

This will increase the corresponding version number by 1.