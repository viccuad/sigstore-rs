//
// Copyright 2021 The Sigstore Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This crate aims to provide [Sigstore](https://www.sigstore.dev/) capabilities to Rust developers.
//!
//! Currently, the main focus of the crate is to provide the verification
//! capabilities offered by the official `cosign` tool.
//!
//! **Warning:** this library is still experimental. Its API can change at any time.
//!
//! # Security
//!
//! Should you discover any security issues, please refer to
//! Sigstore's [security process](https://github.com/sigstore/community/blob/main/SECURITY.md).
//!
//! # Verification
//!
//! Sigstore verification is done using the [`cosign::Client`](crate::cosign::Client)
//! struct.
//!
//! ## Triangulation of Sigstore signature
//!
//! Given a container image/oci artifact, calculate the location of
//! its cosign signature inside of a registry:
//!
//! ```rust,no_run
//! use crate::sigstore::cosign::CosignCapabilities;
//! use std::fs;
//!
//! #[tokio::main]
//! pub async fn main() {
//!   let auth = &sigstore::registry::Auth::Anonymous;
//!
//!   let mut client = sigstore::cosign::ClientBuilder::default()
//!     .build()
//!     .expect("Unexpected failure while building Client");
//!   let image = "registry-testing.svc.lan/kubewarden/disallow-service-nodeport:v0.1.0";
//!   let (cosign_signature_image, source_image_digest) = client.triangulate(
//!     image,
//!     auth
//!   ).await.expect("Unexpected failure while using triangulate");
//! }
//! ```
//!
//! ## Signature verification
//!
//! Verify the signature of a container image/oci artifact:
//!
//! ```rust,no_run
//! use crate::sigstore::cosign::{
//!     CosignCapabilities,
//!     filter_signature_layers,
//! };
//! use crate::sigstore::cosign::verification_constraint::{
//!     AnnotationVerifier,
//!     PublicKeyVerifier,
//!     VerificationConstraintVec,
//! };
//! use crate::sigstore::crypto::SignatureDigestAlgorithm;
//! use crate::sigstore::errors::SigstoreError;
//!
//! use std::boxed::Box;
//! use std::collections::HashMap;
//! use std::fs;
//!
//! #[tokio::main]
//! pub async fn main() {
//!   let auth = &sigstore::registry::Auth::Anonymous;
//!
//!   // Provide both rekor and fulcio data -> this enables keyless verification
//!   // Read rekor's key from the location generated by `cosign initialize`
//!   let rekor_pub_key = fs::read_to_string("~/.sigstore/root/targets/rekor.pub")
//!     .expect("Cannot read rekor public key");
//!   // Read fulcio's certificate from the location generated by `cosign initialize`
//!   let fulcio_cert = fs::read_to_string("~/.sigstore/root/targets/fulcio.crt.pem")
//!     .expect("Cannot read fulcio certificate");
//!
//!   let mut client = sigstore::cosign::ClientBuilder::default()
//!     .with_rekor_pub_key(&rekor_pub_key)
//!     .with_fulcio_cert(fulcio_cert.as_bytes())
//!     .build()
//!     .expect("Unexpected failure while building Client");
//!
//!   // Obtained via `triangulate`
//!   let cosign_image = "registry-testing.svc.lan/kubewarden/disallow-service-nodeport:sha256-5f481572d088dc4023afb35fced9530ced3d9b03bf7299c6f492163cb9f0452e.sig";
//!   // Obtained via `triangulate`
//!   let source_image_digest = "sha256-5f481572d088dc4023afb35fced9530ced3d9b03bf7299c6f492163cb9f0452e";
//!
//!   // Obtain the list the signatures layers associated that can be trusted
//!   let signature_layers = client.trusted_signature_layers(
//!     auth,
//!     cosign_image,
//!     source_image_digest,
//!   ).await.expect("Could not obtain signature layers");
//!
//!   // Define verification constraints
//!   let mut annotations: HashMap<String, String> = HashMap::new();
//!   annotations.insert("env".to_string(), "prod".to_string());
//!   let annotation_verifier = AnnotationVerifier{
//!     annotations,
//!   };
//!
//!   let verification_key = fs::read("~/cosign.pub")
//!     .expect("Cannot read contents of cosign public key");
//!   let pub_key_verifier = PublicKeyVerifier::new(
//!     &verification_key,
//!     SignatureDigestAlgorithm::default(),
//!   ).expect("Could not create verifier");
//!
//!   let verification_constraints: VerificationConstraintVec = vec![
//!     Box::new(annotation_verifier),
//!     Box::new(pub_key_verifier),
//!   ];
//!
//!   // Filter all the trusted layers, find the ones satisfying the constraints
//!   // we just defined
//!   let signatures_matching_requirements = filter_signature_layers(
//!     &signature_layers,
//!     verification_constraints);
//!
//!   match signatures_matching_requirements {
//!     Err(SigstoreError::SigstoreNoVerifiedLayer) => {
//!       panic!("no signature is matching the requirements")
//!     },
//!     Err(e) => {
//!       panic!("Something went wrong while verifying the image: {}", e)
//!     },
//!     Ok(signatures) => {
//!       println!("signatures matching the requirements:");
//!       serde_json::to_writer_pretty(
//!           std::io::stdout(),
//!           &signatures,
//!       ).unwrap();
//!     }
//!   }
//! }
//! ```
//!
//! ## Fulcio and Rekor integration
//!
//! [`cosign::Client`](crate::cosign::Client) integration with Fulcio and Rekor
//! requires the following data to work: Fulcio's certificate and Rekor's public key.
//!
//! These files are safely distributed by the Sigstore project via a TUF repository.
//! The [`sigstore::tuf`](crate::tuf) module provides the helper structures to deal
//! with it.
//!
//! # Examples
//!
//! Additional examples can be found inside of the [`examples`](https://github.com/sigstore/sigstore-rs/tree/main/examples/)
//! directory.

pub mod crypto;
mod mock_client;

pub mod cosign;
pub mod errors;
pub mod registry;
pub mod simple_signing;
pub mod tuf;
