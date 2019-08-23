#pragma once
#include <cstdint>

struct NextPageUrl
{
    const char* url = nullptr;
    int32_t counter = 0;
    int32_t coincidenceCounter = 0;
};

//postID: joyreactor post id
//url: post url
//tags: ([tag](url) [tag](url))
//type: 0 - text
//      1 - image
//      2 - document
//      3 - url
//text: just text
//data: not text :) (may be nullptr)

extern "C" bool get_page_content(const char* baseUrl, //address of the page 
                                 const char* html,
                                 bool(*newReactorUrlCallback)(int64_t postId, const char* url, const char* tags, void* userData),
                                 bool(*newReactorDataCallback)(int64_t postId, int32_t type, const char* text, const char* data, void* userData),
                                 NextPageUrl *nextPageUrl,
                                 void *userData,
                                 bool verbose
                                );
extern "C" void get_page_content_cleanup(NextPageUrl *nextPageUrl);
