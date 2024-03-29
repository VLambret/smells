Feature: Smells social complexity

	Scenario: Analyse a non-git repository
		Given project is not a git repository
		And file.rs is created
		When smells is called with "."
		#Fail because project is in smells
#		Then exit code is 1
#		And the warning "Analysed folder is not a git repository" is raised
		Then no social_complexity metric is computed

	Scenario: Analyse a git repository without any contributors
		Given project is a git repository
		And file.rs is created
		And there is no contributor
		When smells is called with "."
		Then exit code is 0
		Then no warning is raised
		And no social_complexity metric is computed

	Scenario: Analyse a git repository with contributors
		Given project is a git repository
        And author1 add a line to lib/mod1/file1.rs
		And author1 add a line to lib/mod1/file2.rs
		And author2 add a line to lib/mod1/file2.rs
        And author3 add a line to lib/README
		When smells is called with "."
       	Then exit code is 0
		And no warning is raised
		And lib/mod1/file1.rs social_complexity score is 1
		And lib/mod1/file2.rs social_complexity score is 2
		And lib/README social_complexity score is 1
		And lib/mod1 social_complexity score is 2
		And lib social_complexity score is 3


	Scenario: Analyse of a not versioned file in a git repository gives no social complexity score
		Given project is a git repository
		And author1 add a line to file1.rs
		And file2.rs is created
		When smells is called with "."
		Then exit code is 0
		And no warning is raised
		And file1.rs social_complexity score is 1
		And file2.rs has no social_complexity score

	Scenario: Analyse of a subfolder of a git repository
		Given project is a git repository
		And author1 add a line to folder1/file1.rs
		When smells is called with "./folder1"
		Then exit code is 0
		And no warning is raised
		And folder1 social_complexity score is 1
		And folder1/file1.rs social_complexity score is 1
