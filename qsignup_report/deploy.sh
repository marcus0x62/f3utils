#!/bin/bash

set -eo pipefail

if [ ! -e .bucket-name.txt ]; then
    BUCKET_ID=$(dd if=/dev/random bs=8 count=1 2>/dev/null | od -An -tx1 | tr -d ' \t\n')
    BUCKET_NAME=qsignups-report--$BUCKET_ID
    echo $BUCKET_NAME > .bucket-name.txt
    aws s3 mb s3://$BUCKET_NAME
fi

ARTIFACT_BUCKET=$(cat .bucket-name.txt)
PASSWORD=$(cat .db-password.txt)

aws cloudformation package --template-file template.yml --s3-bucket $ARTIFACT_BUCKET \
    --output-template-file .out.yml

perl -spi -e "s/DB_PASSWORD/$PASSWORD/g" .out.yml

aws cloudformation deploy --template-file .out.yml --stack-name nodejs-apig \
    --capabilities CAPABILITY_NAMED_IAM

FUNCTION=$(aws cloudformation describe-stack-resource --stack-name nodejs-apig --logical-resource-id \
               function --query 'StackResourceDetail.PhysicalResourceId' --output text)
APIID=$(aws cloudformation describe-stack-resource --stack-name nodejs-apig --logical-resource-id api \
            --query 'StackResourceDetail.PhysicalResourceId' --output text)
REGION=$(aws configure get region)

echo -n "*** Created/updated Lambda function $FUNCTION, which can be reached at "
echo "https://$APIID.execute-api.$REGION.amazonaws.com/api/" | tee .api-gw-endpoint.txt

echo $FUNCTION > .function-name.txt
