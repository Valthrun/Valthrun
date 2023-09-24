git_commit_shash=$(git rev-parse --short "$GITHUB_SHA")
git_branch=$(echo $GITHUB_REF | cut -d'/' -f 3)
curl -H 'Content-Type: multipart/form-data' \
  -X POST \
  -F "payload_json={\"content\": \"Automated release build for $git_commit_shash on $git_branch\"}" \
  -F "file=@target/release/controller.exe; filename=controller_$git_commit_shash.exe" $DISCORD_RELEASE_WEBHOOK