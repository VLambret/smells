Feature: Smells basic usages

  Scenario: Smells without arguments
    Given no program argument is provided
    When smells is called
    Then exit code is 1
    And standard output is empty
    And standard error contains "USAGE:"

  Scenario: Smells with two arguments
    Given arguments are "folder1 folder2"
    When smells is called
    Then exit code is 1
    And standard output is empty
    And standard error contains "USAGE:"

  Scenario: Smells called with non existing folder
    Given arguments are "non_existing_folder"
    When smells is called
    Then exit code is 1
    And standard output is empty
    And standard error contains "No such file or directory"

  Scenario: Smells nominal case
    Given arguments are "existing_folder"
    When smells is called
    Then exit code is 0
    And standard output is not empty
    And "standard error" is empty

  Scenario: Smells help can be called with long version
    Given arguments are "--help"
    When smells is called
    Then exit code is 0
    And "standard output" contains "USAGE:"
    And "standard error" is empty

  Scenario: Smells help with short version
    Given arguments are "-h"
    When smells is called
    Then exit code is 0
    And "standard output" contains "USAGE:"
    And "standard error" is empty
