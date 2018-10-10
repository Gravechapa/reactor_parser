#pragma once
#include <cstdint>

struct NextPageUrl
{
    const char* url = nullptr;
    int32_t counter = 0;
    int32_t coincidenceCounter = 0;
};

extern "C" bool get_page_content(const char* html, bool(*newReactorUrlCallback)(int64_t, const char*, const char*), bool(*newReactorDataCallback)(int64_t, int32_t, const char*, const char*), NextPageUrl *nextPageUrl);
extern "C" void get_page_content_cleanup(NextPageUrl *nextPageUrl);
