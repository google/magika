// Copyright 2026 Google LLC
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

package magika

import (
	"os"
	"testing"
)

func TestReadContentTypesKB(t *testing.T) {
	assetsDir := os.Getenv("MAGIKA_ASSETS_DIR")
	if assetsDir == "" {
		assetsDir = "../../assets"
	}
	ckb, err := readContentTypesKB(assetsDir)
	if err != nil {
		t.Fatalf("readContentTypesKB(%q) failed: %v", assetsDir, err)
	}
	if len(ckb) == 0 {
		t.Fatal("readContentTypesKB returned empty map")
	}

	var heuristicsCount, resolversCount, mlModelCount, vetoedCount int
	for label, ct := range ckb {
		if ct.Label != label {
			t.Errorf("entry %q has mismatched Label %q", label, ct.Label)
		}
		if ct.VetoedByResolvers && !ct.InResolvers {
			t.Errorf("entry %q has VetoedByResolvers=true with InResolvers=false", label)
		}
		if ct.InHeuristics {
			heuristicsCount++
		}
		if ct.InResolvers {
			resolversCount++
		}
		if ct.InMLModel {
			mlModelCount++
		}
		if ct.VetoedByResolvers {
			vetoedCount++
		}
	}

	if heuristicsCount == 0 || resolversCount == 0 || mlModelCount == 0 || vetoedCount == 0 {
		t.Errorf("expected non-zero coverage counts, got heuristics=%d resolvers=%d ml_model=%d vetoed=%d",
			heuristicsCount, resolversCount, mlModelCount, vetoedCount)
	}

	for _, tc := range []struct {
		label             string
		inHeuristics      bool
		inResolvers       bool
		inMLModel         bool
		vetoedByResolvers bool
	}{
		{label: "txt", inHeuristics: true, inResolvers: false, inMLModel: true, vetoedByResolvers: false},
		{label: "empty", inHeuristics: true, inResolvers: false, inMLModel: false, vetoedByResolvers: false},
		{label: "apk", inHeuristics: false, inResolvers: true, inMLModel: true, vetoedByResolvers: false},
		{label: "parquet", inHeuristics: false, inResolvers: true, inMLModel: true, vetoedByResolvers: true},
		{label: "3dsm", inHeuristics: false, inResolvers: true, inMLModel: false, vetoedByResolvers: true},
		{label: "python", inHeuristics: false, inResolvers: false, inMLModel: true, vetoedByResolvers: false},
		{label: "3ds", inHeuristics: false, inResolvers: false, inMLModel: false, vetoedByResolvers: false},
	} {
		ct, ok := ckb[tc.label]
		if !ok {
			t.Errorf("missing expected KB label %q", tc.label)
			continue
		}
		if ct.InHeuristics != tc.inHeuristics ||
			ct.InResolvers != tc.inResolvers ||
			ct.InMLModel != tc.inMLModel ||
			ct.VetoedByResolvers != tc.vetoedByResolvers {
			t.Errorf("KB[%q] coverage = (heuristics=%v, resolvers=%v, ml=%v, vetoed=%v), want (%v, %v, %v, %v)",
				tc.label,
				ct.InHeuristics, ct.InResolvers, ct.InMLModel, ct.VetoedByResolvers,
				tc.inHeuristics, tc.inResolvers, tc.inMLModel, tc.vetoedByResolvers)
		}
	}
}
