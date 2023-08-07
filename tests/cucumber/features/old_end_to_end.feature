Feature: End-to-End feature

  Scenario: If I give smells an empty file in a folder it will analyse the file
    Given a path tests/data/single_file_folder
    When I run the analysis of the folder
    Then smells will show the json result of the analysis
    """
    {
        "single_file_folder": {
            "metrics": {
                "lines_count": 0,
                "social_complexity": 0
            },
            "folder_content_analyses": [
                {
                    "file0.txt": {
                        "metrics": {
                            "lines_count": 0,
                            "social_complexity": 0
                        }
                    }
                }
            ]
        }
    }
    """

    Scenario: If I give smells multiple files in a folder it will analyse the files
        Given a path tests/data/folder_with_multiple_files
        When I run the analysis of the folder
        Then smells will show the json result of the analysis
        """
        {
            "folder_with_multiple_files":{
                "metrics": {
                    "lines_count": 6,
                    "social_complexity": 0
                },
                "folder_content_analyses":[
                    {
                    "file1.txt": {
                        "metrics": {
                            "lines_count": 1,
                            "social_complexity": 0
                        }
                    }
                    },
                    {
                    "file5.txt": {
                        "metrics": {
                            "lines_count": 5,
                            "social_complexity": 0
                        }
                    }
                    }
                ]
            }
        }
        """

    Scenario: If I give smells an empty folder in a folder it will give empty metrics
        Given a path tests/data/folder_with_one_empty_folder
        When I run the analysis of the folder
        Then smells will show the json result of the analysis
        """
        {
            "folder_with_one_empty_folder":{
                "metrics": {
                    "lines_count": null,
                    "social_complexity": null
                },
                "folder_content_analyses":[]
            }
        }
        """

      Scenario: If I give smells a file in one folder AND a file in a subfolder
          Given a path tests/data/folder_with_folder_and_file
          When I run the analysis of the folder
          Then smells will show the json result of the analysis
          """
          {
            "folder_with_folder_and_file": {
                "metrics": {
                    "lines_count": 11,
                    "social_complexity": 0
                },
            "folder_content_analyses": [
                {
                    "file1.txt": {
                        "metrics": {
                            "lines_count": 1,
                            "social_complexity": 0
                        }
                    }
                },
                {
                    "folder": {
                        "metrics": {
                            "lines_count": 10,
                            "social_complexity": 0
                        },
                    "folder_content_analyses": [
                    {
                        "file10.txt": {
                            "metrics": {
                                "lines_count": 10,
                                "social_complexity": 0
                            }
                        }
                    }]
                    }
                }]
            }
          }
          """