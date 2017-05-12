cmake_minimum_required(VERSION 2.8)
project(cabal-extract)

add_custom_target(cabal-extract ALL cargo build
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
        SOURCES ${CMAKE_CURRENT_SOURCE_DIR}/src/main.rs)