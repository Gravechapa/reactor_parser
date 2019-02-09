use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use std::boxed::Box;

extern crate kuchiki;
use kuchiki::traits::*;
use kuchiki::NodeRef;
use kuchiki::ParseOpts;

#[macro_use] extern crate lazy_static;

extern crate regex;
use regex::Regex;

extern crate url;
use url::Url;

extern crate percent_encoding;
use percent_encoding::percent_decode;

#[derive(Debug)]
enum ElementType
{
    TEXT,
    IMG,
    DOCUMENT,
    URL,
}

impl ElementType {
    fn value(&self) -> i32
    {
        match *self
            {
                ElementType::TEXT => 0,
                ElementType::IMG => 1,
                ElementType::DOCUMENT => 2,
                ElementType::URL => 3,
            }
    }
}

#[repr(C)]
pub struct NextPageUrl
{
    url: *mut c_char,
    counter: i32,
    coincidence_counter: i32,
}

#[derive(Debug)]
struct RawElement
{
    element_type: ElementType,
    data: String,
}

static UNIQ_STRING: &str = "/@split@۝┛";

#[no_mangle]
pub extern "C" fn get_page_content_cleanup(next_page_url: *mut NextPageUrl)
{
    unsafe{CString::from_raw((*next_page_url).url);}
}

#[no_mangle]
pub extern "C" fn get_page_content(html: *const c_char,
                                   new_reactor_url_callback: Option<extern "C" fn(i64, *const c_char, *const c_char) -> bool>,
                                   new_reactor_data_callback: Option<extern "C" fn(i64, i32, *const c_char, *const c_char) -> bool>,
                                   next_page_url: *mut NextPageUrl) -> bool
{
    let safe_new_reactor_data_callback = new_reactor_data_callback.expect("Data callback is NULL");

    let mut check = true;

    let html = unsafe{CStr::from_ptr(html).to_str().unwrap()};

    let mut options = ParseOpts::default();
    options.on_parse_error = Some(Box::new(|err| println!("Parse issue: {}", err)));

    let document = kuchiki::parse_html_with_options(options).one(html);

    for post in document.select(".postContainer").unwrap()
        {
            let post_link = match post.as_node().select_first("a.link[href]")
                {
                    Ok(result) => result,
                    Err(_) => {
                        println!("Can't find post link node");
                        check = false;
                        continue
                    }

                };
            let post_url = match post_link.attributes.borrow().get("href")
                {
                    Some(result) => result.to_string(),
                    None =>  {
                        println!("Can't find \"href\" attribute in post link node");
                        check = false;
                        continue
                    }
                };
            let post_id = &post_url[post_url.rfind('/').unwrap() + 1..].parse::<i64>().unwrap();

            let tags = get_post_tags(post.as_node());

            let result = new_reactor_url_callback.expect("Url callback is NULL")
                (post_id.clone(),
                 CString::new(post_url).unwrap().as_ref().as_ptr(),
                 CString::new(tags).unwrap().as_ref().as_ptr());

            if !result
                {
                    if !next_page_url.is_null()
                        {
                            unsafe {(*next_page_url).coincidence_counter += 1};
                        }
                }
            else
                {
                    let post_content = match post.as_node().select_first(".post_content")
                        {
                            Ok(result) => result,
                            Err(_) => {
                                println!("Can't find post content node, post id: {}", post_id);
                                check = false;
                                continue
                            }
                        };
                    let raw_elements = get_post_content(post_content.as_node(),
                                                                        &post_id,);
                    let post_text = post_content.text_contents();
                    let splitted_text: Vec<&str> = post_text.split(UNIQ_STRING).collect();

                    if raw_elements.is_empty()
                        {
                            let trimmed_text = splitted_text[0].trim();
                            if !trimmed_text.is_empty()
                                {
                                    safe_new_reactor_data_callback(*post_id, ElementType::TEXT.value(),
                                                              CString::new(trimmed_text).unwrap().as_ref().as_ptr(),
                                                              ptr::null());

                                }
                        }
                    else
                        {
                            assert!(raw_elements.len() <= splitted_text.len(),
                                    "Something went wrong with element-text merging");

                            let mut text = String::new();
                            for i in 0..raw_elements.len()
                                {
                                    text.push_str(splitted_text[i]);
                                    if raw_elements[i].element_type.value() == ElementType::TEXT.value()
                                        {
                                            text.push_str(&raw_elements[i].data);
                                        }
                                    else
                                        {
                                            safe_new_reactor_data_callback(*post_id,
                                                                      raw_elements[i].element_type.value(),
                                                                      CString::new(text.trim())
                                                                          .unwrap().as_ref().as_ptr(),
                                                                      CString::new(raw_elements[i]
                                                                          .data.to_string()).unwrap().as_ref().as_ptr());
                                            text = String::new();
                                        }
                                }
                            for i in raw_elements.len()..splitted_text.len()
                                {
                                    text.push_str(splitted_text[i]);
                                }

                            let trimmed_text = text.trim();
                            if !trimmed_text.is_empty()
                                {
                                    safe_new_reactor_data_callback(*post_id,
                                                              ElementType::TEXT.value(),
                                                              CString::new(trimmed_text).unwrap().as_ref().as_ptr(),
                                                              ptr::null());
                                }

                        }
                    if !next_page_url.is_null()
                        {
                            unsafe {(*next_page_url).counter += 1}
                        }
                }
        }

    if !next_page_url.is_null()
        {
            match document.select_first("a.next[href]")
                {
                    Ok(next_page_node) => {
                        let next_page_link = next_page_node.attributes.borrow()
                            .get("href").unwrap().to_string();
                        unsafe {(*next_page_url).url = CString::new(next_page_link).unwrap().into_raw();}
                    },
                    Err(_) => ()
                };
        }
    return check;
}

fn get_post_content(post_content: &NodeRef, post_id: &i64) -> Vec<RawElement>
{
    let mut raw_elements = Vec::<RawElement>::new();

    for trash in post_content.
        select("a.more_link, span.more_content, div.mainheader, div.blog_results, div.post_poll_holder, script").unwrap()
        {
            trash.as_node().detach();
        }

    for element in post_content.
        select(".image > .prettyPhotoLink, .image > img, .image > span.video_gif_holder,\
         .image > iframe[src], a[href]:not([class])").unwrap()
        {
            if element.name.local.eq("a")
                {
                    if element.attributes.borrow().get("class") == Some("prettyPhotoLink")
                        {
                            let link = element.attributes.borrow().get("href").unwrap().to_string();
                            lazy_static!
                            {
                                static ref GIF_CHECKER: Regex = Regex::new("([^\\s]+(\\.(?i)(gif))$)").unwrap();
                            }
                            if GIF_CHECKER.is_match(&link)
                                {
                                    raw_elements.push(RawElement{element_type: ElementType::DOCUMENT,
                                                                        data: link});
                                }
                            else
                                {
                                    raw_elements.push(RawElement{element_type: ElementType::IMG,
                                        data: link});
                                }
                            element.as_node().append(NodeRef::new_text(UNIQ_STRING));
                        }
                    else
                        {
                            lazy_static!
                            {
                                static ref REACTOR_REDIRECT_CHECKER: Regex =
                                 Regex::new("^https?://([-a-zA-Z0-9%_]*\\.reactor\\.cc|joyreactor\\.cc|reactor\\.cc)/redirect\\?url=.*").unwrap();
                                static ref URL_CHECKER: Regex =
                                 Regex::new("^(https?|ftp|file)://[-a-zA-Z0-9+&@#/%?=~_|!:,.;]*[-a-zA-Z0-9+&@#/%=~_|]").unwrap();
                            }

                            let mut redirect_url = element.attributes.borrow().get("href").unwrap().to_string();
                            if REACTOR_REDIRECT_CHECKER.is_match(&redirect_url)
                                {
                                    redirect_url = percent_decode(redirect_url[redirect_url.find("url=").unwrap() + 4..].as_ref())
                                        .decode_utf8().unwrap().to_string()
                                }

                            match element.as_node().first_child()
                                {
                                    Some(text_node) => {
                                        let text = match text_node.as_text()
                                            {
                                                Some(result) => result.borrow().to_string(),
                                                None => {
                                                    println!("Can't find text in url node: {}", post_id);
                                                    continue
                                                }
                                            };
                                        if URL_CHECKER.is_match(&text)
                                            {
                                                redirect_url = format!("\"{}\"", redirect_url);
                                            }
                                        else
                                            {
                                                redirect_url = format!("{} \"{}\"", text , redirect_url);
                                            }

                                        text_node.detach();
                                        },
                                    None => {redirect_url = format!("\"{}\"", redirect_url);}
                                };

                            raw_elements.push(RawElement{element_type: ElementType::TEXT, data: redirect_url});
                            element.as_node().append(NodeRef::new_text(UNIQ_STRING));
                        }
                }
            if element.name.local.eq("img")
                {
                    let link = element.attributes.borrow().get("src").unwrap().to_string();
                    raw_elements.push(RawElement{element_type: ElementType::IMG, data: link});
                    element.as_node().append(NodeRef::new_text(UNIQ_STRING));
                }
            if element.name.local.eq("span") && element.attributes.borrow().get("class") == Some("video_gif_holder")
                {
                    let gif = match element.as_node().select_first(".video_gif_source")
                        {
                            Ok(result) => result,
                            Err(_) => continue
                        };
                    gif.as_node().first_child().unwrap().detach();
                    gif.as_node().append(NodeRef::new_text(UNIQ_STRING));

                    raw_elements.push(RawElement{element_type: ElementType::DOCUMENT,
                        data: gif.attributes.borrow().get("href").unwrap().to_string()});
                }
            if element.name.local.eq("iframe") && element.attributes.borrow().get("src").is_some()
                {
                    let mut link = element.attributes.borrow().get("src").unwrap().to_string();
                    let link_url = match Url::parse(&link)
                        {
                            Ok(result) => result,
                            Err(_) => continue
                        };
                    let domain = link_url.domain().unwrap();
                    let path = link_url.path();

                    if domain.eq("www.coub.com") || domain.eq("coub.com")
                        {
                            link = "https://www.coub.com/view".to_string() + &path[path.rfind("/").unwrap()..];
                        }
                    else if domain.eq("www.youtube.com") || domain.eq("youtube.com")
                        {
                            link = "https://www.youtube.com/watch?v=".to_string() + &path[path.rfind("/").unwrap() + 1..];
                        }
                    raw_elements.push(RawElement{element_type: ElementType::URL, data: link});
                    element.as_node().append(NodeRef::new_text(UNIQ_STRING));
                }
        }
    for new_line in post_content.select("br, h3, h4, h5, h6").unwrap()
        {
            new_line.as_node().append(NodeRef::new_text("\n"));
        }
    return raw_elements;
}

fn get_post_tags(post: &NodeRef) -> String
{
    let mut tags = String::new();
    let tags_list = match post.select_first(".taglist")
        {
            Ok(result) => result,
            Err(_) => return tags
        };
    for tags_link in tags_list.as_node().select("a[href]").unwrap()
        {
            tags += &format!("[{}]({}) ", &tags_link.as_node().text_contents(),
                           &tags_link.attributes.borrow().get("href").unwrap());
        }
    return tags;
}
