Feature: analysis feature

  Scenario: If I analyse an empty folder without metrics
    Given an empty folder without metrics
    When I do the internal analysis
    Then analysis module will build this analysis
    """
    { "file_name": "root", "metrics": {}, "folder_content": {} }
    """

  Scenario: If I analyse a two files folder without metrics
    Given a two files folder without metrics
    When I do the internal analysis
    Then analysis module will build this analysis
    """
    {
        "file_name": "root",
        "metrics": {},
        "folder_content": {
            "file1": {
                "file_name": "file1",
                "metrics": {},
                "folder_content": null
            },
            "file2": {
                "file_name": "file2",
                "metrics": {},
                "folder_content": null
            }
        }
    }
    """

