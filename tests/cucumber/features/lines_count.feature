Feature: Smells lines count

  #Could be in basic usage tests
  Scenario: Analyse an empty project
    Given an empty project
    When smells is called with "."
    Then exit code is 1
	And the warning "Analysed folder is empty" is raised
    And standard output is empty

#  Scenario: Analyse an empty file
#    Given project is created
#    And file0.rs is created
#    And 0 lines are added to file0.rs
#    When smells is called with "."
#    Then exit code is 0
#    And no warning is raised
#    And no lines_count metric is computed
#
#  Scenario: Analyse of an unauthorized file
#
#  Scenario: Analyse of folder ending with slash
#
#  Scenario: Analyse non empty files in folders
#    Given project is created
#    And lib/mod1/file2.rs is created
#    And lib/mod1/file5.rs is created
#    And lib/file8.rs is created
#    And 2 lines are added to lib/mod1/file2.rs
#    And 5 lines are added to lib/mod1/file5.rs
#    And 8 lines are added to lib/file8.rs
#    When smells is called with "."
#    Then exit code is 0
#    And no warning is raised
#    And lib/mod1/file2.rs lines_count score is 2
#    And lib/mod1/file5.rs lines_count score is 5
#    And lib/file8.rs lines_count score is 8
#    And lib/mod1 lines_count score is 7
#    And lib lines_count score is 15


