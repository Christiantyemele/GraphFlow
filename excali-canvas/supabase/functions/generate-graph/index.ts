import { serve } from "https://deno.land/std@0.190.0/http/server.ts";
import { createClient } from "https://esm.sh/@supabase/supabase-js@2.57.2";

const corsHeaders = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Headers": "authorization, x-client-info, apikey, content-type",
};

const GRAPHFLOW_API_URL = "http://localhost:5173";

serve(async (req) => {
  if (req.method === "OPTIONS") {
    return new Response(null, { headers: corsHeaders });
  }

  const supabaseClient = createClient(
    Deno.env.get("SUPABASE_URL") ?? "",
    Deno.env.get("SUPABASE_ANON_KEY") ?? ""
  );

  try {
    const authHeader = req.headers.get("Authorization")!;
    const token = authHeader.replace("Bearer ", "");
    const { data: userData } = await supabaseClient.auth.getUser(token);
    const user = userData.user;
    if (!user) throw new Error("User not authenticated");

    const { content, tier, allow_images, assets_dir } = await req.json();
    console.log("Generate graph request:", { contentLength: content?.length, tier });

    if (!content) {
      throw new Error("Content is required");
    }

    // Call GraphFlow API
    const response = await fetch(`${GRAPHFLOW_API_URL}/graph/generate`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        content,
        tier,
        allow_images,
        assets_dir,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error("GraphFlow API error:", response.status, errorText);
      throw new Error(`GraphFlow API failed: ${response.status} - ${errorText}`);
    }

    const data = await response.json();
    console.log("Graph generated successfully");

    return new Response(
      JSON.stringify(data),
      {
        headers: { ...corsHeaders, "Content-Type": "application/json" },
        status: 200,
      }
    );
  } catch (error) {
    console.error("Error in generate-graph function:", error);
    return new Response(
      JSON.stringify({
        error: error instanceof Error ? error.message : "Failed to generate graph",
      }),
      {
        headers: { ...corsHeaders, "Content-Type": "application/json" },
        status: 500,
      }
    );
  }
});
