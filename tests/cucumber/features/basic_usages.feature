Feature: Smells basic usages

  Scenario: Smells without arguments
    When smells is called with ""
    Then exit code is 1
    And standard output is empty
    And standard error contains "USAGE:"

  Scenario: Smells with two arguments
    When smells is called with "folder1 folder2"
    Then exit code is 1
    And standard output is empty
    And standard error contains "USAGE:"

  Scenario: Smells called with non existing folder
    When smells is called with "./non_existing_folder"
    Then exit code is 1
    And standard output is empty
    And standard error contains "No such file or directory"

  Scenario: Smells called with an empty project
    Given a project
    And the project is empty
    When smells is called with "."
    Then exit code is 10
    And the warning "Analysed folder does not contain any file" is raised
    And standard output is empty

  Scenario: Smells called with a project containing an empty folder
    Given a project
    And lib/file.rs is created
    And file lib/file.rs is deleted
    When smells is called with "."
    Then exit code is 10
    And the warning "Analysed folder does not contain any file" is raised
    And standard output is empty

  Scenario: Smells ignores hidden files
    Given a project
    And lib/.mod/file1.rs is created
    And lib/.file2.rs is created
    When smells is called with "."
    Then exit code is 10
    And the warning "Analysed folder does not contain any file" is raised
    And standard output is empty

  Scenario: Smells nominal case
    Given project is a git repository
    And existing_folder/file0.rs is created
    When smells is called with "./existing_folder"
    Then exit code is 0
    And standard output is not empty
    And standard error is empty

  @ignore
  Scenario: Smells can filter files to analyse
    Given project is a git repository
    And lib/file.rs is created
    And lib/file.c is created
    And lib/file.h is created
    When smells is called with ". --filter c,h"
    Then exit code is 0
    And standard error is empty
    And lib/file.c lines_count score is 0
    And lib/file.h lines_count score is 0
    And lib/file.rs is not included in analysis


  Scenario: Smells help can be called with long version
    When smells is called with "--help"
    Then exit code is 0
    And standard output contains "USAGE:"
    And standard error is empty

  Scenario: Smells help with short version
    When smells is called with "-h"
    Then exit code is 0
    And standard output contains "USAGE:"
    And standard error is empty
