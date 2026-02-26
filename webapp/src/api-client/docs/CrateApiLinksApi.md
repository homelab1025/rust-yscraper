# CrateApiLinksApi

All URIs are relative to *http://localhost*

|Method | HTTP request | Description|
|------------- | ------------- | -------------|
|[**deleteLink**](#deletelink) | **DELETE** /links/{id} | |
|[**listLinks**](#listlinks) | **GET** /links | Retrieve all links with their item IDs and added date.|
|[**scrapeLink**](#scrapelink) | **POST** /scrape | Triggers scraping and inserts results into the database. Trigger a scrape task for a specific Hacker News item|

# **deleteLink**
> deleteLink()


### Example

```typescript
import {
    CrateApiLinksApi,
    Configuration
} from './api';

const configuration = new Configuration();
const apiInstance = new CrateApiLinksApi(configuration);

let id: number; //Link ID to delete (default to undefined)

const { status, data } = await apiInstance.deleteLink(
    id
);
```

### Parameters

|Name | Type | Description  | Notes|
|------------- | ------------- | ------------- | -------------|
| **id** | [**number**] | Link ID to delete | defaults to undefined|


### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
|**200** | Link deleted successfully |  -  |
|**404** | Link not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **listLinks**
> Array<LinkDto> listLinks()


### Example

```typescript
import {
    CrateApiLinksApi,
    Configuration
} from './api';

const configuration = new Configuration();
const apiInstance = new CrateApiLinksApi(configuration);

const { status, data } = await apiInstance.listLinks();
```

### Parameters
This endpoint does not have any parameters.


### Return type

**Array<LinkDto>**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
|**200** | List of all links |  -  |
|**500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **scrapeLink**
> ScrapeResponse scrapeLink(scrapeRequest)


### Example

```typescript
import {
    CrateApiLinksApi,
    Configuration,
    ScrapeRequest
} from './api';

const configuration = new Configuration();
const apiInstance = new CrateApiLinksApi(configuration);

let scrapeRequest: ScrapeRequest; //

const { status, data } = await apiInstance.scrapeLink(
    scrapeRequest
);
```

### Parameters

|Name | Type | Description  | Notes|
|------------- | ------------- | ------------- | -------------|
| **scrapeRequest** | **ScrapeRequest**|  | |


### Return type

**ScrapeResponse**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
|**200** | Scrape task scheduled or already scheduled |  -  |
|**500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

