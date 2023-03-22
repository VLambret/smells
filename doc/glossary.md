{
    "folder_with_multiple_files":{
        "metrics": {
            "lines_metric": 6
        },
        "folder_content":[
            {
            "file1.txt": {
                "metrics": {
                    "lines_metric": 1
                }
            }
            },
            {
            "file5.txt": {
                "metrics": {
                    "lines_metric": 5
                }
            }
            }
        ]
    }
}

## Metric

```
    "lines_metric": 6
```

The key is the type of the **metric**, and the content is its **value**

## Metrics

```
"metrics": {
            "lines_metric": 6
        }
```

The set of metrics related to an **analyzed item**

## Analyzed item

```
"folder_with_multiple_files":{
        "metrics": {
            "lines_metric": 6
        },
        "folder_content":[
            {
            "file1.txt": {
                "metrics": {
                    "lines_metric": 1
                }
            }
            },
            {
            "file5.txt": {
                "metrics": {
                    "lines_metric": 5
                }
            }
            }
        ]
    }
```

This program can run analysis on both files and folder.
- For **files**, metrics are computed directly
- For **folders**, at the moment, the metrics are a **summary** of their content

## File

```
"file1.txt": {
    "metrics": {
        "lines_metric": 1
    }
}
```

When the **analysed item** is a file.
- The **key** is the base name of the file
- The content is an object containing only **metrics**

## Folder

```
    "folder_with_multiple_files":{
        "metrics": {
            "lines_metric": 6
        },
        "folder_content":[
            {
            "file1.txt": {
                "metrics": {
                    "lines_metric": 1
                }
            }
            },
            {
            "file5.txt": {
                "metrics": {
                    "lines_metric": 5
                }
            }
            }
        ]
    }
```


When the **analysed item** is a folder.
- The **key** is the base name of the folder
- The content is an object containing:
    - the summary of the **metrics**
    - The **folder content**


## Folder content

```
        "folder_content":[
            {
            "file1.txt": {
                "metrics": {
                    "lines_metric": 1
                }
            }
            },
            {
            "file5.txt": {
                "metrics": {
                    "lines_metric": 5
                }
            }
            }
        ]
```

The folder content is an element containing an array with an analysed item for each of the files contained in the folder
