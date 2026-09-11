import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "resolve-published-release.sh"


class ReleaseResolutionTests(unittest.TestCase):
    def resolve(self, *, event="workflow_run", tag="v0.3.0", prerelease=False,
                result="success", upstream="push", repository="owner/repo",
                draft=False, manual_ref="refs/heads/main"):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            gh = root / "gh"
            gh.write_text("""#!/usr/bin/env python3
import json, os, pathlib, sys
if sys.argv[1:3] == ['run', 'download']:
    assert sys.argv[3] == '123'
    pathlib.Path('plan').mkdir()
    pathlib.Path('plan/plan-dist-manifest.json').write_text(os.environ['PLAN'])
elif sys.argv[1:3] == ['release', 'view']:
    print(os.environ['RELEASE'])
else:
    sys.exit(99)
""")
            gh.chmod(0o755)
            output = root / "output"
            env = dict(os.environ, PATH=f"{root}{os.pathsep}{os.environ['PATH']}",
                       EVENT_NAME=event, REQUESTED_TAG=tag,
                       RELEASE_RUN_EVENT=upstream, RELEASE_RUN_RESULT=result,
                       RELEASE_RUN_REPOSITORY=repository,
                       RELEASE_RUN_ID="123", GITHUB_REPOSITORY="owner/repo",
                       GITHUB_REF=manual_ref, GITHUB_OUTPUT=str(output),
                       PLAN=json.dumps({"announcement_tag": tag,
                                        "announcement_is_prerelease": prerelease}),
                       RELEASE=json.dumps({"tagName": tag, "isDraft": draft,
                                           "isPrerelease": prerelease}))
            process = subprocess.run(["bash", str(SCRIPT)], cwd=root, env=env,
                                     capture_output=True, text=True)
            return process.returncode, output.read_text() if output.exists() else ""

    def test_successful_release_resolves_exact_run_tag(self):
        self.assertEqual(self.resolve(), (0, "tag=v0.3.0\n"))

    def test_prerelease_skips_publication(self):
        self.assertEqual(self.resolve(tag="v0.3.0-beta.1", prerelease=True), (0, ""))

    def test_failed_pr_and_foreign_runs_cannot_publish(self):
        for case in ({"result": "failure"}, {"upstream": "pull_request"},
                     {"repository": "fork/repo"}):
            with self.subTest(case=case):
                code, output = self.resolve(**case)
                self.assertNotEqual(code, 0)
                self.assertEqual(output, "")

    def test_malformed_tags_and_drafts_cannot_publish(self):
        for case in ({"tag": "v01.2.3"}, {"tag": "v1.2.3\ntag=other"},
                     {"tag": "latest"}, {"draft": True}):
            with self.subTest(case=case):
                code, output = self.resolve(**case)
                self.assertNotEqual(code, 0)
                self.assertEqual(output, "")

    def test_manual_retry_requires_main(self):
        self.assertEqual(self.resolve(event="workflow_dispatch"), (0, "tag=v0.3.0\n"))
        code, output = self.resolve(event="workflow_dispatch", manual_ref="refs/heads/other")
        self.assertNotEqual(code, 0)
        self.assertEqual(output, "")


if __name__ == "__main__":
    unittest.main()
