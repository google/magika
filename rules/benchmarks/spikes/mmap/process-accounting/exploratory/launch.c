// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0
#include <errno.h>
#include <fcntl.h>
#include <spawn.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>
extern char **environ;
static uint64_t now(clockid_t clock) {
    struct timespec t;
    if (clock_gettime(clock, &t)) _exit(125);
    return (uint64_t)t.tv_sec * 1000000000 + t.tv_nsec;
}
int main(int argc, char **argv) {
    if (argc < 4) return 125;
    posix_spawn_file_actions_t actions;
    if (posix_spawn_file_actions_init(&actions)) return 125;
    int out = open(argv[1], O_WRONLY|O_CREAT|O_TRUNC, 0600);
    int err = open(argv[2], O_WRONLY|O_CREAT|O_TRUNC, 0600);
    if (out < 0 || err < 0) return 125;
    if (posix_spawn_file_actions_adddup2(&actions, out, STDOUT_FILENO)) return 125;
    if (posix_spawn_file_actions_adddup2(&actions, err, STDERR_FILENO)) return 125;
    if (posix_spawn_file_actions_addclose(&actions, out)) return 125;
    if (posix_spawn_file_actions_addclose(&actions, err)) return 125;
    pid_t child;
    uint64_t start_unix = now(CLOCK_REALTIME), start = now(CLOCK_MONOTONIC);
    int error = posix_spawn(&child, argv[3], &actions, NULL, &argv[3], environ);
    uint64_t spawned = now(CLOCK_MONOTONIC);
    if (error) { errno = error; perror("posix_spawn"); return 125; }
    int status;
    while (waitpid(child, &status, 0) < 0) if (errno != EINTR) return 125;
    uint64_t end = now(CLOCK_MONOTONIC), end_unix = now(CLOCK_REALTIME);
    posix_spawn_file_actions_destroy(&actions);
    close(out); close(err);
    printf("{\"start_unix_ns\":%llu,\"end_unix_ns\":%llu,\"elapsed_ns\":%llu,\"spawn_call_ns\":%llu,\"status\":%d}\n",
           (unsigned long long)start_unix, (unsigned long long)end_unix,
           (unsigned long long)(end-start), (unsigned long long)(spawned-start), status);
    return WIFEXITED(status) ? WEXITSTATUS(status) : 125;
}
