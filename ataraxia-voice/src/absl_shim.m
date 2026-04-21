// Shim for +[NSString stringForAbslStringView:] missing on newer macOS.
//
// The prebuilt libwebrtc uses Abseil, which normally injects this method via
// a compiled-in ObjC category.  On Darwin 25+ the method isn't present at
// runtime, crashing inside VP9 codec enumeration.  This category provides the
// same selector with the correct ABI so the ObjC runtime finds it first.
//
// absl::string_view on AArch64 is {const char* ptr, size_t len} — identical
// to the struct below.

#import <Foundation/Foundation.h>
#include <stddef.h>

struct AbslStringViewABI { const char *ptr; size_t len; };

@interface NSString (AbslStringViewShim)
+ (instancetype)stringForAbslStringView:(struct AbslStringViewABI)sv;
@end

@implementation NSString (AbslStringViewShim)
+ (instancetype)stringForAbslStringView:(struct AbslStringViewABI)sv {
    if (sv.ptr == NULL || sv.len == 0) return @"";
    return [[NSString alloc] initWithBytes:sv.ptr
                                   length:sv.len
                                 encoding:NSUTF8StringEncoding];
}
@end
