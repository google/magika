cmake_minimum_required(VERSION 3.16)

project(MagikaBasicSample LANGUAGES CXX)

option(MAGIKA_ENABLE_WARNINGS "Enable compiler warnings" ON)

add_library(magika_project_options INTERFACE)
target_compile_features(magika_project_options INTERFACE cxx_std_17)

if(MAGIKA_ENABLE_WARNINGS)
  target_compile_options(magika_project_options INTERFACE -Wall -Wextra -Wpedantic)
endif()
