cmake_minimum_required(VERSION 3.26)
project(lampcord_voice LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
set(CMAKE_INTERPROCEDURAL_OPTIMIZATION OFF)
set_property(GLOBAL PROPERTY JOB_POOLS voice_compile=2 voice_link=1)
set(CMAKE_JOB_POOL_COMPILE voice_compile)
set(CMAKE_JOB_POOL_LINK voice_link)
set(CMAKE_CXX_VISIBILITY_PRESET hidden)
set(CMAKE_VISIBILITY_INLINES_HIDDEN ON)
set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
set(BUILD_TESTING OFF CACHE BOOL "" FORCE)
set(TESTING OFF CACHE BOOL "" FORCE)
set(PERSISTENT_KEYS OFF CACHE BOOL "" FORCE)
set(DISABLE_GREASE ON CACHE BOOL "" FORCE)
set(DISABLE_PQ ON CACHE BOOL "" FORCE)
set(OPENSSL_SMALL ON CACHE BOOL "" FORCE)
set(REQUIRE_BORINGSSL ON CACHE BOOL "" FORCE)
set(MLS_CXX_NAMESPACE mlspp CACHE STRING "" FORCE)
set(MLS_EXPORT_NAMESPACE MLSPP CACHE STRING "" FORCE)

# BoringSSL uses NASM on Windows.
if(WIN32 AND CMAKE_SYSTEM_PROCESSOR MATCHES "AMD64|x86_64|amd64|x86|i[3-6]86")
    find_program(NASM nasm REQUIRED)
endif()

include(FetchContent)
# A pinned source bundle of Discord libdave, MLSpp and BoringSSL. No prebuilt crypto binaries.
FetchContent_Declare(dave_deps
    URL https://codeload.github.com/KyokoBot/libdave-jvm/tar.gz/ce725965eb81e4878b474da226ea7cb0053f6cf9
    URL_HASH SHA256=1b1aed8297f54f02a37c5be536fc31202a235a6bd2b93b65b8804c999c8eab11
    SOURCE_SUBDIR unused)
FetchContent_MakeAvailable(dave_deps)
set(DEPS "${dave_deps_SOURCE_DIR}/natives")
# Disable P-256 ADX on Windows/MinGW to avoid missing symbols.
set(p256 "${DEPS}/boringssl/third_party/fiat/p256_64.h")
file(READ "${p256}" p256_source)
if(NOT p256_source MATCHES "OPENSSL_WINDOWS")
    string(REPLACE
        "#if !defined(OPENSSL_NO_ASM) && defined(__GNUC__) && defined(__x86_64__)"
        "#if !defined(OPENSSL_NO_ASM) && defined(__GNUC__) && defined(__x86_64__) && !defined(OPENSSL_WINDOWS)"
        p256_source "${p256_source}")
    file(WRITE "${p256}" "${p256_source}")
endif()
# The bundle's package configs assume they are at the project root.
file(MAKE_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/packages")
foreach(package OpenSSL MLSPP)
    file(READ "${DEPS}/cmake/${package}Config.cmake" config)
    string(REPLACE "\${CMAKE_SOURCE_DIR}" "${DEPS}" config "${config}")
    file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/packages/${package}Config.cmake" "${config}")
    set(${package}_DIR "${CMAKE_CURRENT_BINARY_DIR}/packages")
endforeach()
set(CMAKE_FIND_PACKAGE_PREFER_CONFIG ON)
set(nlohmann_json_DIR "${DEPS}/json-cpp")
# Android's sysroot-only find_path must not hide this bundled host-side source path.
set(BORINGSSL_INCLUDE_DIR "${DEPS}/boringssl/include" CACHE PATH "" FORCE)
add_subdirectory("${DEPS}/boringssl" boringssl EXCLUDE_FROM_ALL)
add_subdirectory("${DEPS}/mlspp" mlspp EXCLUDE_FROM_ALL)
add_subdirectory("${DEPS}/libdave/cpp" libdave EXCLUDE_FROM_ALL)
if(ANDROID)
    # libdave's Android public interface uses JNIEnv without including its declaration.
    target_compile_options(libdave PRIVATE -include jni.h)
endif()

set(OPUS_BUILD_PROGRAMS OFF CACHE BOOL "" FORCE)
set(OPUS_BUILD_TESTING OFF CACHE BOOL "" FORCE)
FetchContent_Declare(opus
    URL https://codeload.github.com/xiph/opus/tar.gz/refs/tags/v1.5.2
    URL_HASH SHA256=9480e329e989f70d69886ded470c7f8cfe6c0667cc4196d4837ac9e668fb7404)
FetchContent_MakeAvailable(opus)

FetchContent_Declare(rnnoise
    URL https://codeload.github.com/xiph/rnnoise/tar.gz/master)
FetchContent_MakeAvailable(rnnoise)

set(RNNOISE_SOURCES
    "${rnnoise_SOURCE_DIR}/src/denoise.c"
    "${rnnoise_SOURCE_DIR}/src/rnn.c"
    "${rnnoise_SOURCE_DIR}/src/rnn_data.c"
    "${rnnoise_SOURCE_DIR}/src/kiss_fft.c"
    "${rnnoise_SOURCE_DIR}/src/celt_lpc.c"
    "${rnnoise_SOURCE_DIR}/src/pitch.c"
)
add_library(rnnoise_static STATIC ${RNNOISE_SOURCES})
target_include_directories(rnnoise_static PUBLIC "${rnnoise_SOURCE_DIR}/include")
target_compile_definitions(rnnoise_static PRIVATE HAVE_CONFIG_H)
# RNNoise expects a config.h. We can provide a minimal one or just define the needed macros.
file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/config.h" "
#define PACKAGE_NAME \"rnnoise\"
#define PACKAGE_VERSION \"0.1\"
#define TRAIN_SET 1
")
target_include_directories(rnnoise_static PRIVATE "${CMAKE_CURRENT_BINARY_DIR}")
target_compile_options(rnnoise_static PRIVATE -w) # Suppress warnings from 3rd party code

add_library(lampcord_voice SHARED voice.cpp)
target_link_libraries(lampcord_voice PRIVATE libdave opus OpenSSL::Crypto rnnoise_static)
if(NOT ANDROID)
    find_package(JNI REQUIRED)
    target_include_directories(lampcord_voice PRIVATE ${JNI_INCLUDE_DIRS})
    add_executable(voice_check EXCLUDE_FROM_ALL voice_check.cpp "${DEPS}/libdave/cpp/test/external_sender.cpp")
    target_include_directories(voice_check PRIVATE ${JNI_INCLUDE_DIRS} "${DEPS}/libdave/cpp/test" "${DEPS}/libdave/cpp/src")
    target_link_libraries(voice_check PRIVATE libdave opus OpenSSL::Crypto MLSPP::mlspp)
else()
    target_link_options(lampcord_voice PRIVATE "-Wl,-z,max-page-size=16384")
endif()
set_target_properties(lampcord_voice PROPERTIES
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/out"
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/out")
foreach(config Release Debug RelWithDebInfo MinSizeRel)
    string(TOUPPER "${config}" upper)
    set_target_properties(lampcord_voice PROPERTIES
        LIBRARY_OUTPUT_DIRECTORY_${upper} "${CMAKE_CURRENT_BINARY_DIR}/out"
        RUNTIME_OUTPUT_DIRECTORY_${upper} "${CMAKE_CURRENT_BINARY_DIR}/out")
endforeach()

# Include dependency notices with both the desktop and Android distributions.
file(MAKE_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/licenses")
configure_file("${DEPS}/libdave/LICENSE" "licenses/libdave.txt" COPYONLY)
configure_file("${DEPS}/mlspp/LICENSE" "licenses/mlspp.txt" COPYONLY)
configure_file("${DEPS}/boringssl/LICENSE" "licenses/boringssl.txt" COPYONLY)
configure_file("${DEPS}/json-cpp/LICENSE.MIT" "licenses/nlohmann-json.txt" COPYONLY)
configure_file("${opus_SOURCE_DIR}/COPYING" "licenses/opus.txt" COPYONLY)
