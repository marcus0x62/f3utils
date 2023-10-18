#!/bin/bash

set -eo pipefail

if [ ! -e .bucket-name.txt ]; then
    BUCKET_ID=$(dd if=/dev/random bs=8 count=1 2>/dev/null | od -An -tx1 | tr -d ' \t\n')
    BUCKET_NAME=fngbot-$BUCKET_ID
    echo $BUCKET_NAME > .bucket-name.txt
    aws s3 mb s3://$BUCKET_NAME
fi

ARTIFACT_BUCKET=$(cat .bucket-name.txt)

aws cloudformation package --template-file template.yml --s3-bucket $ARTIFACT_BUCKET \
    --output-template-file .out.yml

. secrets.env

perl -spi -e "s/SLACK_API_KEY/$SLACK_API_KEY/g" .out.yml
perl -spi -e "s/MAILCHIMP_API_KEY/$MAILCHIMP_API_KEY/g" .out.yml
perl -spi -e "s/SLACK_SIGNING_SECRET/$SLACK_SIGNING_SECRET/g" .out.yml

aws cloudformation deploy --template-file .out.yml --stack-name fngbot \
    --capabilities CAPABILITY_NAMED_IAM

FUNCTION=$(aws cloudformation describe-stack-resource --stack-name fngbot --logical-resource-id \
               function --query 'StackResourceDetail.PhysicalResourceId' --output text)
APIID=$(aws cloudformation describe-stack-resource --stack-name fngbot --logical-resource-id api \
            --query 'StackResourceDetail.PhysicalResourceId' --output text)
REGION=$(aws configure get region)

echo -n "*** Created/updated Lambda function $FUNCTION, which can be reached at "
echo "https://$APIID.execute-api.$REGION.amazonaws.com/api/" | tee .api-gw-endpoint.txt

echo $FUNCTION > .function-name.txt
