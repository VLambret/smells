Feature: Smells combined metrics analysis

  Scenario: Analyse a git repository with multiple contributors, folders and files
    Given project is a git repository
    And file0.rs is created
    And author1 add a line to lib/mod1/file1.rs
    And author1 add a line to lib/mod1/file2.rs
    And author2 add a line to lib/mod1/file2.rs
    And author3 add a line to lib/README
    When smells is called with "."
    Then exit code is 0
    And no warning is raised
    And file0.rs has no social_complexity score
    # line_score is O or null ?
    And file0.rs lines_count score is 0
    And lib/mod1/file1.rs social_complexity score is 1
    And lib/mod1/file1.rs lines_count score is 1
    And lib/mod1/file2.rs social_complexity score is 2
    And lib/mod1/file2.rs lines_count score is 2
    And lib/README social_complexity score is 1
    And lib/README lines_count score is 1
    And lib/mod1 social_complexity score is 2
    And lib/mod1 lines_count score is 3
    And lib social_complexity score is 3
    And lib lines_count score is 4
