artifact="$1"
file_path="$2"
pdb_path="$3"

file_name=$(basename -- "$file_path")

git_commit_shash=$(git rev-parse --short "$GITHUB_SHA")
git_branch=$(echo $GITHUB_REF | cut -d'/' -f 3)

artifact_track="nightly"
version="$git_branch"
if [ "$git_branch" = "release" ]; then
    artifact_track="release"

    if [[ -z "$ARTIFACT_VERSION" ]]; then
        echo "Missing ARTIFACT_VERSION env var"
        exit 1
    fi

    version="$ARTIFACT_VERSION"
fi

echo "Uploading $file_path"
curl -H "Content-Type:multipart/form-data" \
    -X POST \
    -F "info={\"version\": \"$version\", \"versionHash\": \"$git_commit_shash\", \"updateLatest\": true }" \
    -F "payload=@$file_path; filename=${artifact}_${git_commit_shash}.${file_name##*.}" \
    "https://valth.run/api/artifacts/$artifact/$artifact_track?api-key=$ARTIFACT_API_KEY" || exit 1

echo "Uploading $pdb_path"
curl -H "Content-Type:multipart/form-data" \
    -X POST \
    -F "info={\"version\": \"$version\", \"versionHash\": \"$git_commit_shash\", \"updateLatest\": true }" \
    -F "payload=@$pdb_path; filename=$(basename -- "$pdb_path")" \
    "https://valth.run/api/artifacts/$artifact/$artifact_track-pdb?api-key=$ARTIFACT_API_KEY" || exit 1