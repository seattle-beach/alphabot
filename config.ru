require "json"

require "faraday"
require "roda"

class AlphaBot < Roda
  plugin :json
  plugin :json_parser

  route do |r|
    @conn = Faraday.new("https://slack.com/api/chat.postMessage") do |faraday|
      faraday.headers["Content-Type"] = "application/json"
      faraday.headers["Authorization"] = "Bearer #{ENV["ACCESS_TOKEN"]}"
      faraday.adapter Faraday.default_adapter
    end

    r.post "" do
      p r.params

      raise "Invalid verification token" unless r.params["token"] == ENV["VERIFICATION_TOKEN"]

      case r.params["type"]
      when "url_verification"
        {challenge: r.params["challenge"]}
      when "event_callback"
        event = r.params["event"]
        break {} if event.has_key?("thread_ts")

        args = {
          channel: event["channel"],
          text: "You might want to post this to <#C07BYTE1W|sea-labs>. This channel isn't widely used by the office.",
          thread_ts: event["ts"]
        }
        res = @conn.post "", args.to_json
      else
        raise "Unexpected type: #{r.params["type"]}"
      end

      {}
    end
  end
end

run AlphaBot.freeze.app
