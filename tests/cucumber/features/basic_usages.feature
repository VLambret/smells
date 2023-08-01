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
    When smells is called with "non_existing_folder"
    Then exit code is 1
    And standard output is empty
    And standard error contains "No such file or directory"
#
#  Fail because analysed folder is not a git repo
##  Scenario: Smells nominal case
##    When smells is called with "tests/data/existing_folder"
##    Then exit code is 0
##    And standard output is "not empty"
##    And standard error is empty
#

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
