Feature: Smells metrics analysis

  Scenario: Smells called with a project containing an empty folder
    Given a project
    And the folder lib is created
    When smells is called with "."
    Then exit code is 0
    And standard error is empty
    And standard output is not empty

