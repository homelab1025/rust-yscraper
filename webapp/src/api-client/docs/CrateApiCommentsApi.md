# CrateApiCommentsApi

All URIs are relative to *http://localhost*

|Method | HTTP request | Description|
|------------- | ------------- | -------------|
|[**listComments**](#listcomments) | **GET** /comments | List comments with pagination|
|[**updateCommentState**](#updatecommentstate) | **PATCH** /comments/{id}/state | Update comment state|

# **listComments**
> CommentsPage listComments()


### Example

```typescript
import {
    CrateApiCommentsApi,
    Configuration
} from './api';

const configuration = new Configuration();
const apiInstance = new CrateApiCommentsApi(configuration);

let urlId: number; // (default to undefined)
let offset: number; // (optional) (default to undefined)
let count: number; // (optional) (default to undefined)
let state: CommentState; // (optional) (default to undefined)
let sortBy: SortBy; // (optional) (default to undefined)
let sortOrder: SortOrder; // (optional) (default to undefined)

const { status, data } = await apiInstance.listComments(
    urlId,
    offset,
    count,
    state,
    sortBy,
    sortOrder
);
```

### Parameters

|Name | Type | Description  | Notes|
|------------- | ------------- | ------------- | -------------|
| **urlId** | [**number**] |  | defaults to undefined|
| **offset** | [**number**] |  | (optional) defaults to undefined|
| **count** | [**number**] |  | (optional) defaults to undefined|
| **state** | **CommentState** |  | (optional) defaults to undefined|
| **sortBy** | **SortBy** |  | (optional) defaults to undefined|
| **sortOrder** | **SortOrder** |  | (optional) defaults to undefined|


### Return type

**CommentsPage**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
|**200** | List of comments |  -  |
|**500** | Database error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **updateCommentState**
> updateCommentState(updateStateRequest)


### Example

```typescript
import {
    CrateApiCommentsApi,
    Configuration,
    UpdateStateRequest
} from './api';

const configuration = new Configuration();
const apiInstance = new CrateApiCommentsApi(configuration);

let id: number; //Comment ID (default to undefined)
let updateStateRequest: UpdateStateRequest; //

const { status, data } = await apiInstance.updateCommentState(
    id,
    updateStateRequest
);
```

### Parameters

|Name | Type | Description  | Notes|
|------------- | ------------- | ------------- | -------------|
| **updateStateRequest** | **UpdateStateRequest**|  | |
| **id** | [**number**] | Comment ID | defaults to undefined|


### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
|**200** | Comment state updated |  -  |
|**500** | Database error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

