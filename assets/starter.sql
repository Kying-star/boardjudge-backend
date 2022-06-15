CREATE DATABASE `boardjudge`;

USE `boardjudge`;

CREATE TABLE `contest` (
  `id` uuid NOT NULL DEFAULT uuid(),
  `nick` varchar(64) COLLATE utf8mb4_unicode_ci NOT NULL,
  `description` text COLLATE utf8mb4_unicode_ci NOT NULL,
  `start` datetime NOT NULL,
  `end` datetime NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `privilege` (
  `id` uuid NOT NULL DEFAULT uuid(),
  `user_id` uuid NOT NULL,
  `contest_id` uuid NOT NULL,
  `kind` varchar(32) COLLATE utf8mb4_unicode_ci NOT NULL,
  PRIMARY KEY (`id`),
  KEY `privilege_contest_id` (`contest_id`),
  KEY `privilege_user_id` (`user_id`),
  CONSTRAINT `privilege_contest_id` FOREIGN KEY (`contest_id`) REFERENCES `contest` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT `privilege_user_id` FOREIGN KEY (`user_id`) REFERENCES `user` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `problem` (
  `id` uuid NOT NULL DEFAULT uuid(),
  `nick` varchar(64) COLLATE utf8mb4_unicode_ci NOT NULL,
  `description` text COLLATE utf8mb4_unicode_ci NOT NULL,
  `limit_time` int(10) unsigned NOT NULL,
  `limit_memory` int(10) unsigned NOT NULL,
  `contest_id` uuid NOT NULL,
  PRIMARY KEY (`id`),
  KEY `problem_contest_id` (`contest_id`),
  CONSTRAINT `problem_contest_id` FOREIGN KEY (`contest_id`) REFERENCES `contest` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `record` (
  `id` uuid NOT NULL DEFAULT uuid(),
  `time` datetime NOT NULL,
  `user_id` uuid NOT NULL,
  `problem_id` uuid NOT NULL,
  `code` text COLLATE utf8mb4_unicode_ci NOT NULL,
  `language` varchar(32) COLLATE utf8mb4_unicode_ci NOT NULL,
  `result` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL CHECK (json_valid(`result`)),
  `status` varchar(32) COLLATE utf8mb4_unicode_ci NOT NULL,
  PRIMARY KEY (`id`),
  KEY `record_problem_id` (`problem_id`),
  KEY `record_user_id` (`user_id`),
  CONSTRAINT `record_problem_id` FOREIGN KEY (`problem_id`) REFERENCES `problem` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT `record_user_id` FOREIGN KEY (`user_id`) REFERENCES `user` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `user` (
  `id` uuid NOT NULL DEFAULT uuid(),
  `name` varchar(32) CHARACTER SET ascii NOT NULL,
  `nick` varchar(64) COLLATE utf8mb4_unicode_ci NOT NULL,
  `description` text COLLATE utf8mb4_unicode_ci NOT NULL,
  `password` binary(32) NOT NULL,
  `banned` tinyint(1) NOT NULL DEFAULT 0,
  `root` tinyint(1) NOT NULL DEFAULT 0,
  PRIMARY KEY (`id`),
  UNIQUE KEY `UNIQUE` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
