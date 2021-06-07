// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use sc_executor_common::{
	runtime_blob::RuntimeBlob,
	wasm_runtime::WasmModule,
};
use sc_runtime_test::wasm_binary_unwrap;
use codec::{Encode as _, Decode as _};
use std::sync::Arc;

type HostFunctions = sp_io::SubstrateHostFunctions;

struct RuntimeBuilder {
	fast_instance_reuse: bool,
	canonicalize_nans: bool,
	heap_pages: u32,
}

impl RuntimeBuilder {
	fn new_on_demand() -> Self {
		Self {
			fast_instance_reuse: false,
			canonicalize_nans: false,
			heap_pages: 1024,
		}
	}

	fn canonicalize_nans(&mut self, canonicalize_nans: bool) {
		self.canonicalize_nans = canonicalize_nans;
	}

	fn build(self) -> Arc<dyn WasmModule> {
		let blob = RuntimeBlob::uncompress_if_needed(&wasm_binary_unwrap()[..])
			.expect("failed to create a runtime blob out of test runtime");

		let rt = crate::create_runtime(
			blob,
			crate::Config {
				heap_pages: self.heap_pages,
				allow_missing_func_imports: true,
				cache_path: None,
				semantics: crate::Semantics {
					fast_instance_reuse: self.fast_instance_reuse,
					stack_depth_metering: false,
					canonicalize_nans: self.canonicalize_nans,
				},
			},
			{
				use sp_wasm_interface::HostFunctions as _;
				HostFunctions::host_functions()
			}
		)
		.expect("cannot create runtime");

		Arc::new(rt) as Arc<dyn WasmModule>
	}
}

#[test]
fn test_nan_canonicalization() {
	let runtime = {
		let mut builder = RuntimeBuilder::new_on_demand();
		builder.canonicalize_nans(true);
		builder.build()
	};

	let instance = runtime
		.new_instance()
		.expect("failed to instantiate a runtime");

	/// A NaN with canonical payload bits.
	const CANONICAL_NAN_BITS: u32 = 0x7fc00000;
	/// A NaN value with an abitrary payload.
	const ARBITRARY_NAN_BITS: u32 = 0x7f812345;

	// This test works like this: we essentially do
	//
	//     a + b
	//
	// where
	//
	//     * a is a nan with arbitrary bits in its payload
	//     * b is 1.
	//
	// according to the wasm spec, if one of the inputs to the operation is a non-canonical NaN
	// then the value be a NaN with non-deterministic payload bits.
	//
	// However, with the `canonicalize_nans` option turned on above, we expect that the output will
	// be a canonical NaN.
	//
	// We exterpolate the results of this tests so that we assume that all intermediate computations
	// that involve floats are sanitized and cannot produce a non-deterministic NaN.

	let params = (u32::to_le_bytes(ARBITRARY_NAN_BITS), u32::to_le_bytes(1)).encode();
	let res = {
		let raw_result = instance.call_export(
			"test_fp_f32add",
			&params,
		).unwrap();
		u32::from_le_bytes(<[u8; 4]>::decode(&mut &raw_result[..]).unwrap())
	};
	assert_eq!(res, CANONICAL_NAN_BITS);
}
