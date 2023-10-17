SHELL=/bin/bash

build: 
	cd function && make

deploy: .bucket-name.txt
	ARTIFACT_BUCKET=$(shell cat .bucket-name.txt)
	aws cloudformation package --template-file template.yml --s3-bucket $(ARTIFACT_BUCKET) --output-template-file .out.yml
	aws cloudformation deploy --template-file .out.yml --stack-name fngbot --capabilities CAPABILITY_NAMED_IAM

	FUNCTION=$(shell aws cloudformation describe-stack-resource --stack-name fngbot --logical-resource-id \
               function --query 'StackResourceDetail.PhysicalResourceId' --output text)
	APIID=$(shell aws cloudformation describe-stack-resource --stack-name fngbot --logical-resource-id api \
            --query 'StackResourceDetail.PhysicalResourceId' --output text)
	REGION=$(shell aws configure get region)

	echo -n "*** Created/updated Lambda function $(FUNCTION), which can be reached at "
	echo "https://$(APIID).execute-api.$(REGION).amazonaws.com/api/" | tee .api-gw-endpoint.txt

	echo $(FUNCTION) > .function-name.txt

.bucket-name.txt:
	test -e $@ || (export BUCKET_ID=$$(dd if=/dev/random bs=8 count=1 2>/dev/null | od -An -tx1 | tr -d ' \t\n'); export BUCKET_NAME=fngbot-$$BUCKET_ID; echo $$BUCKET_NAME > .bucket-name.txt; aws s3 mb s3://$$BUCKET_NAME)
