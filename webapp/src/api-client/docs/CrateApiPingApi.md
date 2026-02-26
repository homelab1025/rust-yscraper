# CrateApiPingApi

All URIs are relative to *http://localhost*

|Method | HTTP request | Description|
|------------- | ------------- | -------------|
|[**ping**](#ping) | **GET** /ping | Health check handler: echoes back the provided &#x60;msg&#x60; with the current Unix timestamp|

# **ping**
> PingResponse ping()


### Example

```typescript
import {
    CrateApiPingApi,
    Configuration
} from './api';

const configuration = new Configuration();
const apiInstance = new CrateApiPingApi(configuration);

let msg: string; //Message to echo back (optional) (default to undefined)

const { status, data } = await apiInstance.ping(
    msg
);
```

### Parameters

|Name | Type | Description  | Notes|
|------------- | ------------- | ------------- | -------------|
| **msg** | [**string**] | Message to echo back | (optional) defaults to undefined|


### Return type

**PingResponse**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, text/plain


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
|**200** | Ping successful |  -  |
|**400** | Missing required query parameter: msg |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

