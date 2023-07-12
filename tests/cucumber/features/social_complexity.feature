Feature: Smells social complexity

	Scenario: Analyse a non-git repository
		Given analysed folder is not a git repository
#		When smells is called
#		Then exit code is 0
#		And the warning "Analysed folder is not a git repository" is raised
#		And no social complexity metric is computed

#	Scenario: Analyse a git repository without any contributors
#		Given analysed folder is a git repository
#		And there is no contributor
#		When smells is called
#		Then exit code is 0
#		And no warning is raised
#		And no social complexity metric is computed
#
#	Scenario: Analyse a git repository with contributors
#		Given analysed folder is a git repository
#        And "author1" contributed to "lib/mod1/file1.rs"
#        And "author1" contributed to "lib/mod1/file2.rs"
#        And "author2" contributed to "lib/mod1/file2.rs"
#        And "author3" contributed to "lib/README"
#       	When smells is called
#        Then exit code is 0
#		And no warning is raised
#		And "lib/mod1/file1.rs" social complexity score is 1
#		And "lib/mod1/file2.rs" social complexity score is 2
#		And "lib/README" social complexity score is 1
#		And "lib/mod1" social complexity score is 2
#		And "lib" social complexity score is 3