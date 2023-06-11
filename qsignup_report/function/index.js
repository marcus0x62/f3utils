/*
 * index.js -- QSignups Report Lambda Function
 * Created: Marcus Butler <marcusb@marcusb.org>, 10-June-2023.
 */

let debug = false;

exports.handler = async function(event, context) {
    const mysql = require('mysql2/promise');
    const { SSMClient, GetParameterCommand } = require("@aws-sdk/client-ssm");
    
    let team_id = '';
    let start_date = '';
    let stop_date = '';
    let now = new Date();

    debug = process.env.debug;

    log('## ENVIRONMENT VARIABLES: ' + serialize(process.env));
    log('## CONTEXT: ' + serialize(context));
    log('## EVENT: ' + serialize(event));

    let db_password = '';
    const ssm_client = new SSMClient({region: 'us-east-2'});
    const ssm_cmd = new GetParameterCommand({Name: "/qsignup_report/dbPassword", WithDecryption: true});
    const ssm_res = await ssm_client.send(ssm_cmd);
    if (ssm_res?.Parameter) {
        db_password = ssm_res.Parameter.Value;
        log('Successfully loaded DB password from SSM');
    }
    
    try {
        const dbconn = await mysql.createConnection({
            host: process.env.dbHost,
            database: process.env.dbDatabase,
            user: process.env.dbUsername,
            password: db_password
        });

        if (has(event['queryStringParameters'], 'team_id')) {
            team_id = event['queryStringParameters']['team_id'];
        } else {
            return formatResponse("Error: please provide team_id parameter", 500);
        }

        if (has(event['queryStringParameters'], 'start_date')) {
            start_date = event['queryStringParameters']['start_date'];
        } else {
            start_date = `${now.getFullYear()}-${now.getMonth() + 1}-${now.getDate()}`;
            log (`Constructing start_date... ${start_date}`);
        }

        if (has(event['queryStringParameters'], 'stop_date')) {
	    stop_date = event['queryStringParameters']['stop_date'];
	} else {
            stop_date = `${now.getFullYear()}-${now.getMonth() + 2}-${now.getDate()}`;
            log(`Constructing stop date.... ${stop_date}`);
        }

        let query = `SELECT ao.ao_display_name, qm.event_date, qm.event_special, qm.q_pax_name
                     FROM f3stcharles.qsignups_aos ao
                     LEFT JOIN f3stcharles.qsignups_master qm ON (ao.ao_channel_id = qm.ao_channel_id)
                     WHERE qm.team_id = ? AND qm.event_date >= ? AND qm.event_date <= ?
                     ORDER BY qm.event_date ASC;`;

        const [rows, fields] = await dbconn.execute(query, [team_id, start_date, stop_date]);
        log(rows);

        return formatResponse(serialize(rows));
    } catch(error) {
        return formatResponse(error.code  + ": " + error.message, 500);
    }
}

var has = function(obj, key) {
    try {
        len = obj[key].length; // Make sure obj[key] exists and is a string.
    } catch(error) {
        return false;
    }
    return true;
}

var formatResponse = function(body, code=200){
  var response = {
    "statusCode": code,
    "headers": {
        "Content-Type": "application/json",
        "Access-Control-Allow-Origin": "*"
    },
    "isBase64Encoded": false,
    "body": body
  }
  return response
}

var serialize = function(object) {
  return JSON.stringify(object, null, 2)
}

var log = function(message) {
    if (debug == true) {
        console.log(message);
    }
}
