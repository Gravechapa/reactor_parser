#include "../headers/RustReactorParser.h"
#include <fstream>
#include <sstream>
#include <iostream>

bool newReactorUrl(int64_t id, const char* url, const char* tags)
{
    //std::cout << "UC " << id << " " << url << " " << tags << std::endl;
    return 1;
}

bool newReactorData(int64_t id, int32_t type, const char* text, const char* data)
{
    std::cout << "DC " << id << " " << type << " " << text << " " << (data ? data : "") << std::endl;
    return 1;
}

int main()
{
    std::ifstream testFile("test.html");
    if(!testFile.is_open())
    {
        throw "Can't open file test.html";
    }
    std::stringstream buffer;
    buffer << testFile.rdbuf();
    NextPageUrl nextPageUrl;
    get_page_content(buffer.str().c_str(), &newReactorUrl, &newReactorData, &nextPageUrl);
    std::cout << (nextPageUrl.url? nextPageUrl.url : "") << " " << nextPageUrl.counter << " " << nextPageUrl.coincidenceCounter << std::endl;
    get_page_content_cleanup(&nextPageUrl);
    
    std::ifstream testFile1("single_post_test.html");
    if(!testFile1.is_open())
    {
        throw "Can't open file single_post_test.html";
    }
    buffer.str("");
    buffer << testFile1.rdbuf();
    get_page_content(buffer.str().c_str(), &newReactorUrl, &newReactorData, nullptr);
}
