Feature: Analysis feature

  Scenario: If I give smells an empty file in a folder it will analyse the file
    Given a folder with an empty file
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