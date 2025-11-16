import { serve } from "https://deno.land/std@0.190.0/http/server.ts";
import { createClient } from "https://esm.sh/@supabase/supabase-js@2.57.2";

const corsHeaders = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Headers": "authorization, x-client-info, apikey, content-type",
};

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

    const { messages, type } = await req.json();
    console.log("Generate request:", { type, messageCount: messages.length });

    const LOVABLE_API_KEY = Deno.env.get("LOVABLE_API_KEY");
    if (!LOVABLE_API_KEY) throw new Error("LOVABLE_API_KEY not configured");

    // Handle image generation for "picture" type
    if (type === "picture") {
      console.log("Generating image...");
      const response = await fetch("https://ai.gateway.lovable.dev/v1/chat/completions", {
        method: "POST",
        headers: {
          Authorization: `Bearer ${LOVABLE_API_KEY}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          model: "google/gemini-2.5-flash-image-preview",
          messages: [
            {
              role: "system",
              content: "You are an expert at creating stunning visual images based on descriptions. Generate high-quality, detailed images that match the user's request. For modifications, analyze the previous image and the requested changes."
            },
            ...messages
          ],
          modalities: ["image", "text"]
        }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        console.error("Image generation error:", response.status, errorText);
        throw new Error(`Image generation failed: ${response.status}`);
      }

      const data = await response.json();
      console.log("Image generated successfully");
      
      return new Response(
        JSON.stringify({
          type: "image",
          content: data.choices?.[0]?.message?.content || "",
          imageUrl: data.choices?.[0]?.message?.images?.[0]?.image_url?.url || null
        }),
        {
          headers: { ...corsHeaders, "Content-Type": "application/json" },
          status: 200,
        }
      );
    }

    // Handle diagram generation for other types (flow, sequence, mindmap, etc.)
    console.log("Generating diagram...");
    const response = await fetch("https://ai.gateway.lovable.dev/v1/chat/completions", {
      method: "POST",
      headers: {
        Authorization: `Bearer ${LOVABLE_API_KEY}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        model: "google/gemini-2.5-flash",
        messages: [
          {
            role: "system",
            content: `You are an expert at creating diagrams using Mermaid syntax. Generate clear, well-structured ${type || 'flowchart'} diagrams based on the user's description. For modifications, adjust the existing diagram according to the user's request. Always return valid Mermaid syntax.`
          },
          ...messages
        ]
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error("Diagram generation error:", response.status, errorText);
      throw new Error(`Diagram generation failed: ${response.status}`);
    }

      const aiResponse = await response.json();
      console.log("Diagram generated successfully");

      // Extract Mermaid code from markdown code blocks if present
      let mermaidCode = aiResponse.choices?.[0]?.message?.content || "";
      const codeBlockMatch = mermaidCode.match(/```(?:mermaid)?\n([\s\S]*?)\n```/);
      if (codeBlockMatch) {
        mermaidCode = codeBlockMatch[1];
      }

      return new Response(
        JSON.stringify({
          type: "diagram",
          content: aiResponse.choices?.[0]?.message?.content || "",
          mermaidCode: mermaidCode
        }),
        {
          headers: { ...corsHeaders, "Content-Type": "application/json" },
          status: 200,
        }
      );
    } catch (error) {
      console.error("Error:", error);
      const errorMessage = error instanceof Error ? error.message : "Unknown error";
      return new Response(
        JSON.stringify({ error: errorMessage }),
      {
        headers: { ...corsHeaders, "Content-Type": "application/json" },
        status: 500,
      }
    );
  }
});
