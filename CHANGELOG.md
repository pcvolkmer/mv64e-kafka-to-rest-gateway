# Changelog

## [0.3.0](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/compare/v0.2.0...v0.3.0) (2025-12-15)


### ⚠ BREAKING CHANGES

* update dto lib to version 0.2.0 ([#10](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/issues/10))

### deps

* update dto lib to version 0.2.0 ([#10](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/issues/10)) ([90e017b](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/commit/90e017b0021e26ee61406e6cf8b7f3f9f40931e4))

## [0.2.0](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/compare/v0.1.1...v0.2.0) (2025-11-21)


### ⚠ BREAKING CHANGES

* downgrade of mtb-dto due to delayed update of DNPM:DIP ([#8](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/issues/8))

### Bug Fixes

* downgrade of mtb-dto due to delayed update of DNPM:DIP ([#8](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/issues/8)) ([c1448ce](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/commit/c1448cedd74178974b656bbc904d12630c4dc8d3))

## [0.1.1](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/compare/v0.1.0...v0.1.1) (2025-11-21)


### Features

* accept self signed certs if in dev/debug ([071a1da](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/commit/071a1da90cb5e5affa1b774d76a403c6b9f2ddfb))
* add more logging for HTTP requests ([#2](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/issues/2)) ([4e1e321](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/commit/4e1e3211da2de01e137641de55f806eb8a6ad511))
* add test files ([bd63a29](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/commit/bd63a295a01ad0f3cce8db09816e4d662bdd0ad0))
* check mv consent provision for deny ([fb3d50c](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/commit/fb3d50c4e811d61c04a023d336f54e8b454bdba1))
* delete on missing sequencing provision ([#3](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/issues/3)) ([18fc4ad](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/commit/18fc4ad69f323d46e8cc804da5113d844cf22004))
* do not accept records without request ID ([370da27](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/commit/370da2778879b4cb143ce38880468f4265c0ba84))
* do not commit records with invalid responses from DIP ([f933ae8](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/commit/f933ae8886eadf25c1bd94a877a6fbca4b17d1c9))
* do not use empty username or password ([700eac1](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/commit/700eac107887fa837dc3c505988d0dc5d480b198))
* enable ssl connection to kafka ([4f2eb56](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/commit/4f2eb56b07993fc720eea17c348ca9ef5b0c1b14))
* log error if response could not be sent ([a3eef08](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/commit/a3eef0851096d6275e43cfc9655a523f06845f50))
* use dedicated RootCA für HTTPS connections ([1accf0b](https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway/commit/1accf0ba2123944bc1a66a6ba5ae4a2dbdb4c4d1))
