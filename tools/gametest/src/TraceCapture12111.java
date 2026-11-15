package org.nucleation.oracle;

import java.lang.reflect.*;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
import java.util.function.BooleanSupplier;

/**
 * A small, playerless Minecraft 1.21.11 GameTestServer oracle. Uses the original
 * obfuscated server and official ProGuard mappings directly; no patched server,
 * Fabric, remapper, client or compiler dependency beyond a JDK is required.
 * Captures tick-end states and inventories, not intra-tick notification order.
 */
public final class TraceCapture12111 {
    static final String MC = "net.minecraft.server.MinecraftServer";
    static final String LEVEL = "net.minecraft.world.level.Level";
    static final String SERVER_LEVEL = "net.minecraft.server.level.ServerLevel";
    static final String POS = "net.minecraft.core.BlockPos";
    static final String STATE = "net.minecraft.world.level.block.state.BlockState";
    static final String TAG = "net.minecraft.nbt.CompoundTag";
    static final String PROVIDER = "net.minecraft.core.HolderLookup$Provider";
    static final String ENTITY = "net.minecraft.world.level.block.entity.BlockEntity";
    static final int ORIGIN_Y = 100;
    static Names names;
    static Object server, world, registry, blocks;
    static final Map<String, Object> parsedStates = new HashMap<>();
    record Block(int x, int y, int z, String state, String nbt) {}

    public static void main(String[] args) {
        try { run(args); }
        catch (Throwable error) { error.printStackTrace(); System.exit(1); }
    }

    static void run(String[] args) throws Exception {
        Path mappings = Path.of(arg(args, "--mappings", null));
        Path universe = Path.of(arg(args, "--universe", null));
        Path out = Path.of(arg(args, "--out", null));
        String scenario = arg(args, "--case", "comparator_drain");
        boolean negative = Arrays.asList(args).contains("--negative-control");
        int ticks = Integer.parseInt(arg(args, "--ticks", "40"));
        if (Files.exists(universe)) throw new IllegalArgumentException("Use a new universe path: " + universe);
        if (!Set.of("comparator_drain", "piston_qc").contains(scenario)) throw new IllegalArgumentException(scenario);
        if (ticks < 20) throw new IllegalArgumentException("Capture at least 20 ticks");
        String digest = HexFormat.of().formatHex(MessageDigest.getInstance("SHA-1").digest(Files.readAllBytes(mappings)));
        if (!digest.equals("5621e9253f05fd57872bbe7f8ddf5f9a7d525955")) throw new IllegalArgumentException("Unexpected 1.21.11 mappings: " + digest);
        names = new Names(mappings);
        Path loadedServer = Path.of(names.type("net.minecraft.SharedConstants").getProtectionDomain().getCodeSource().getLocation().toURI());
        String loadedDigest = HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(loadedServer)));
        if (!loadedDigest.equals("ec47239a8de246335e1d54f6ac319bd35641778eb4b6a6da06372840d02fcebc")) throw new IllegalArgumentException("Unexpected loaded server: " + loadedServer + " " + loadedDigest);
        names.call(null, "net.minecraft.SharedConstants", "tryDetectVersion", "");
        names.call(null, "net.minecraft.server.Bootstrap", "bootStrap", "");
        names.call(null, "net.minecraft.util.Util", "startTimerHackThread", "");
        Object storage = names.call(null, "net.minecraft.world.level.storage.LevelStorageSource", "createDefault", "java.nio.file.Path", universe);
        Object access = names.call(storage, "net.minecraft.world.level.storage.LevelStorageSource", "createAccess", "java.lang.String", "gametestworld");
        Object packs = names.call(null, "net.minecraft.server.packs.repository.ServerPacksSource", "createPackRepository",
            "net.minecraft.world.level.storage.LevelStorageSource$LevelStorageAccess", access);
        server = names.call(null, "net.minecraft.gametest.framework.GameTestServer", "create",
            "java.lang.Thread,net.minecraft.world.level.storage.LevelStorageSource$LevelStorageAccess,net.minecraft.server.packs.repository.PackRepository,java.util.Optional,boolean",
            Thread.currentThread(), access, packs, Optional.empty(), false);
        boolean success = false;
        try {
            names.call(server, MC, "initServer", "");
            world = names.call(server, MC, "overworld", "");
            registry = names.call(world, LEVEL, "registryAccess", "");
            blocks = names.call(registry, PROVIDER, "lookupOrThrow", "net.minecraft.resources.ResourceKey",
                names.field("net.minecraft.core.registries.Registries", "BLOCK"));
            int warmup = warmChunks();
            List<Block> fixture = fixture(scenario, negative);
            // Author these small fixtures directly in 1.21.11's native schema.
            // No newer schematic is mislabeled or silently datafixed backwards.
            for (int x = -2; x <= 7; x++) for (int y = -2; y <= 5; y++) for (int z = -2; z <= 2; z++)
                rawState(x, y, z, "minecraft:air");
            for (Block b : fixture) rawState(b.x, b.y, b.z, b.state);
            for (Block b : fixture) if (b.nbt != null) {
                Object tag = compound(b.nbt);
                Object entity = names.call(null, ENTITY, "loadStatic", POS + "," + STATE + "," + TAG + "," + PROVIDER,
                    pos(b.x, b.y, b.z), state(b.state), tag, registry);
                if (entity == null) throw new IllegalStateException("Block entity failed to load: " + b);
                names.call(world, LEVEL, "setBlockEntity", ENTITY, entity);
            }
            Object random = names.call(world, LEVEL, "getRandom", "");
            names.call(random, "net.minecraft.util.RandomSource", "setSeed", "long", 0L);
            Map<String,Object> initial = snapshot(0);
            if (((List<?>) initial.get("blocks")).size() != fixture.size()) throw new IllegalStateException("Initial block count mismatch");
            for (Block b : fixture) {
                if (!descriptor(current(b.x,b.y,b.z)).equals(b.state)) throw new IllegalStateException("Initial state mismatch: " + b);
            }
            List<Map<String,Object>> snapshots = new ArrayList<>();
            for (int tick = 1; tick <= ticks; tick++) {
                if (scenario.equals("piston_qc")) {
                    if (tick == 1 || tick == 12) notifyAt(2, 2, 0);
                    if (tick == 10 && !negative) set(1, 2, 0, "minecraft:air");
                }
                tick();
                snapshots.add(snapshot(tick));
            }
            validate(scenario, negative, initial, snapshots);
            Map<String,Object> result = new LinkedHashMap<>();
            result.put("mc_version", "1.21.11"); result.put("data_version", 4671);
            result.put("case", scenario); result.put("negative_control", negative);
            result.put("origin", List.of(0, ORIGIN_Y, 0)); result.put("rng_seed", 0);
            result.put("size", List.of(6,4,1)); result.put("settle", "in_world");
            result.put("action_boundary", "immediately before the numbered tick; initial snapshot is tick zero");
            List<Object> actions=new ArrayList<>();
            if(scenario.equals("piston_qc")){
                actions.add(action(1,"notify_neighbors",2,2,0,null));
                if(!negative)actions.add(action(10,"set_block",1,2,0,"minecraft:air"));
                actions.add(action(12,"notify_neighbors",2,2,0,null));
            }
            result.put("actions",actions);
            result.put("warmup_ticks", warmup); result.put("capture_ticks", ticks);
            result.put("mappings_sha1", digest); result.put("server_inner_sha256", loadedDigest);
            result.put("java_runtime_version",System.getProperty("java.runtime.version"));
            result.put("server_kind","Minecraft GameTestServer; no client or connected players");
            result.put("fixture", fixture.stream().map(b -> {
                Map<String,Object> row = new LinkedHashMap<>(); row.put("pos",List.of(b.x,b.y,b.z));
                row.put("state",b.state); if (b.nbt != null) row.put("nbt",b.nbt); return row;
            }).toList());
            result.put("initial", initial); result.put("snapshots", snapshots);
            result.put("scope", "tick-end block states, comparator outputs and occupied inventory slots; no intra-tick capture");
            if (out.toAbsolutePath().getParent()!=null) Files.createDirectories(out.toAbsolutePath().getParent());
            Files.writeString(out, json(result) + "\n");
            System.out.println("CAPTURED " + out + " ticks=" + ticks + " players=0 case=" + scenario + " negative=" + negative);
            success = true;
        } finally {
            names.call(server, MC, "stopServer", "");
        }
        // Minecraft's process-wide background executors need not keep this
        // finite capture driver alive after the server has shut down.
        if (success) System.exit(0);
    }

    static List<Block> fixture(String scenario, boolean negative) {
        var b = new ArrayList<Block>();
        b.add(new Block(2,0,0,"minecraft:stone",null));
        b.add(new Block(3,0,0,"minecraft:stone",null));
        if (scenario.equals("piston_qc")) {
            b.add(new Block(2,1,0,"minecraft:piston[extended=false,facing=east]",null));
            b.add(new Block(3,1,0,"minecraft:stone",null));
            if (!negative) b.add(new Block(1,2,0,"minecraft:redstone_block",null));
        } else {
            b.add(new Block(3,1,0,"minecraft:stone",null));
            b.add(new Block(2,1,0,"minecraft:hopper[enabled="+!negative+",facing=down]",
                "{id:\"minecraft:hopper\",TransferCooldown:0}"));
            b.add(new Block(2,2,0,"minecraft:barrel[facing=up,open=false]",
                "{id:\"minecraft:barrel\",Items:[{Slot:0b,id:\"minecraft:redstone\",count:3}]}"));
            b.add(new Block(3,2,0,"minecraft:comparator[facing=west,mode=compare,powered=false]",
                "{id:\"minecraft:comparator\",OutputSignal:0}"));
            if (negative) b.add(new Block(1,1,0,"minecraft:redstone_block",null));
        }
        return b;
    }

    static int itemCount(Map<String,Object> snapshot, int x, int y, int z) {
        int total=0;
        for(Object value:(List<?>)snapshot.get("inventories")){
            List<?> row=(List<?>)value;
            if(row.get(0).equals(x)&&row.get(1).equals(y)&&row.get(2).equals(z))total+=(int)row.get(5);
        }
        return total;
    }
    static Map<String,Object> action(int tick,String type,int x,int y,int z,String state){
        Map<String,Object> row=new LinkedHashMap<>();row.put("tick",tick);row.put("type",type);
        row.put("pos",List.of(x,y,z));if(state!=null)row.put("state",state);return row;
    }
    static String at(Map<String,Object> snapshot,int x,int y,int z){
        for(Object value:(List<?>)snapshot.get("blocks")){
            List<?> row=(List<?>)value;
            if(row.get(0).equals(x)&&row.get(1).equals(y)&&row.get(2).equals(z))return (String)row.get(3);
        }
        return "minecraft:air";
    }
    static void require(boolean yes,String message){if(!yes)throw new IllegalStateException(message);}
    static void validate(String scenario,boolean negative,Map<String,Object> initial,List<Map<String,Object>> snapshots){
        Map<String,Object> end=snapshots.get(snapshots.size()-1);
        if(scenario.equals("comparator_drain")){
            require(itemCount(initial,2,2,0)==3&&itemCount(initial,2,1,0)==0,"Initial inventory was not restored");
            if(negative){
                for(var row:snapshots)require(itemCount(row,2,2,0)==3&&itemCount(row,2,1,0)==0,"Locked hopper control transferred items");
            }else{
                require(itemCount(end,2,2,0)==0&&itemCount(end,2,1,0)==3,"Hopper did not drain the barrel");
                require(snapshots.stream().anyMatch(row->at(row,3,2,0).contains("powered=true")),"Comparator never responded to inventory change");
                require(at(end,3,2,0).contains("powered=false"),"Comparator did not return to zero");
            }
        }else{
            require(at(initial,2,1,0).contains("extended=false"),"QC piston moved during raw placement");
            if(negative){
                require(snapshots.stream().noneMatch(row->at(row,2,1,0).contains("extended=true")),"Unpowered QC control extended");
            }else{
                require(snapshots.stream().anyMatch(row->at(row,2,1,0).contains("extended=true")),"QC piston never extended");
                require(at(snapshots.get(9),2,1,0).contains("extended=true")&&at(snapshots.get(10),2,1,0).contains("extended=true"),"QC piston retracted before explicit notification");
                require(at(end,2,1,0).contains("extended=false"),"QC piston failed to retract after notification");
            }
        }
    }

    static int warmChunks() throws Exception {
        Object cache = names.call(world, SERVER_LEVEL, "getChunkSource", "");
        List<Object> required = new ArrayList<>();
        for (int x=-2;x<=1;x++) for(int z=-2;z<=1;z++) {
            Object chunk = names.construct("net.minecraft.world.level.ChunkPos", "int,int", x,z);
            if(x>=-1 && x<=0 && z>=-1 && z<=0) required.add(chunk);
            for(String ticket: List.of("PLAYER_SIMULATION","FORCED")) names.call(cache,
                "net.minecraft.server.level.ServerChunkCache", "addTicketWithRadius",
                "net.minecraft.server.level.TicketType,net.minecraft.world.level.ChunkPos,int",
                names.field("net.minecraft.server.level.TicketType",ticket),chunk,2);
            names.call(world, SERVER_LEVEL, "setChunkForced", "int,int,boolean",x,z,true);
        }
        for(int n=0;n<600;n++) {
            boolean ready=true;
            for(Object chunk:required) {
                long packed=(long)names.call(chunk,"net.minecraft.world.level.ChunkPos","toLong","");
                ready &= (boolean)names.call(world,SERVER_LEVEL,"isPositionTickingWithEntitiesLoaded","long",packed);
            }
            if(ready)return n;
            tick();
        }
        throw new IllegalStateException("Chunks never reached entity-ticking readiness");
    }

    static void tick() throws Exception {
        names.call(server,MC,"tickServer","java.util.function.BooleanSupplier",(BooleanSupplier)()->true);
        names.call(server,MC,"waitUntilNextTick","");
    }
    static Object pos(int x,int y,int z) throws Exception { return names.construct(POS,"int,int,int",x,y+ORIGIN_Y,z); }
    static Object compound(String snbt) throws Exception { return names.call(null,"net.minecraft.nbt.TagParser","parseCompoundFully","java.lang.String",snbt); }
    static Object state(String descriptor) throws Exception {
        Object result=parsedStates.get(descriptor); if(result!=null)return result;
        int bracket=descriptor.indexOf('['); String id=bracket<0?descriptor:descriptor.substring(0,bracket);
        StringBuilder snbt=new StringBuilder("{Name:\"").append(id).append("\"");
        if(bracket>=0){snbt.append(",Properties:{");boolean first=true;
            for(String entry:descriptor.substring(bracket+1,descriptor.length()-1).split(",")){
                String[] pair=entry.split("=");if(!first)snbt.append(',');first=false;
                snbt.append(pair[0]).append(":\"").append(pair[1]).append('"');
            } snbt.append('}');}
        snbt.append('}');
        result=names.call(null,"net.minecraft.nbt.NbtUtils","readBlockState","net.minecraft.core.HolderGetter,"+TAG,blocks,compound(snbt.toString()));
        parsedStates.put(descriptor,result);return result;
    }
    static Object current(int x,int y,int z) throws Exception {return names.call(world,LEVEL,"getBlockState",POS,pos(x,y,z));}
    static void rawState(int x,int y,int z,String descriptor) throws Exception {
        Object chunk=names.call(world,LEVEL,"getChunk","int,int",x>>4,z>>4);
        int sectionIndex=(int)names.call(world,"net.minecraft.world.level.LevelHeightAccessor","getSectionIndex","int",y+ORIGIN_Y);
        Object section=names.call(chunk,"net.minecraft.world.level.chunk.ChunkAccess","getSection","int",sectionIndex);
        names.call(section,"net.minecraft.world.level.chunk.LevelChunkSection","setBlockState","int,int,int,"+STATE+",boolean",
            x&15,(y+ORIGIN_Y)&15,z&15,state(descriptor),false);
    }
    static void set(int x,int y,int z,String descriptor) throws Exception {
        names.call(world,LEVEL,"setBlock",POS+","+STATE+",int",pos(x,y,z),state(descriptor),3);
    }
    static void notifyAt(int x,int y,int z) throws Exception {
        names.call(world,LEVEL,"updateNeighborsAt",POS+",net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation",
            pos(x,y,z),names.field("net.minecraft.world.level.block.Blocks","AIR"),null);
    }
    static String descriptor(Object state) {
        String s=state.toString();
        if(!s.startsWith("Block{"))throw new IllegalStateException("Unexpected block descriptor: "+s);
        s=s.substring(6).replaceFirst("}","");int bracket=s.indexOf('[');
        if(bracket<0)return s;
        String[] props=s.substring(bracket+1,s.length()-1).split(",");Arrays.sort(props);
        return s.substring(0,bracket)+"["+String.join(",",props)+"]";
    }
    static Map<String,Object> snapshot(int tick) throws Exception {
        int players=(int)names.call(server,MC,"getPlayerCount","");
        if(players!=0)throw new IllegalStateException("Capture requires zero players");
        List<Object> blockRows=new ArrayList<>(), inventories=new ArrayList<>(), comparatorOutputs=new ArrayList<>();
        Object itemRegistry=names.field("net.minecraft.core.registries.BuiltInRegistries","ITEM");
        Class<?> container=names.type("net.minecraft.world.Container");
        Class<?> comparator=names.type("net.minecraft.world.level.block.entity.ComparatorBlockEntity");
        for(int y=-2;y<=5;y++)for(int z=-2;z<=2;z++)for(int x=-2;x<=7;x++){
            Object p=pos(x,y,z);String value=descriptor(names.call(world,LEVEL,"getBlockState",POS,p));
            if(value.equals("minecraft:air"))continue;
            blockRows.add(List.of(x,y,z,value));
            Object be=names.call(world,LEVEL,"getBlockEntity",POS,p);
            if(be!=null && comparator.isInstance(be)) {
                int output=(int)names.call(be,"net.minecraft.world.level.block.entity.ComparatorBlockEntity","getOutputSignal","");
                comparatorOutputs.add(List.of(x,y,z,output));
            }
            if(be!=null && container.isInstance(be)){
                int count=(int)names.call(be,"net.minecraft.world.Container","getContainerSize","");
                for(int slot=0;slot<count;slot++){
                    Object stack=names.call(be,"net.minecraft.world.Container","getItem","int",slot);
                    if((boolean)names.call(stack,"net.minecraft.world.item.ItemStack","isEmpty",""))continue;
                    Object item=names.call(stack,"net.minecraft.world.item.ItemStack","getItem","");
                    String id=names.call(itemRegistry,"net.minecraft.core.Registry","getKey","java.lang.Object",item).toString();
                    int amount=(int)names.call(stack,"net.minecraft.world.item.ItemStack","getCount","");
                    inventories.add(List.of(x,y,z,slot,id,amount));
                }
            }
        }
        Map<String,Object> row=new LinkedHashMap<>();row.put("tick",tick);row.put("players",players);
        row.put("blocks",blockRows);row.put("inventories",inventories);row.put("comparator_outputs",comparatorOutputs);return row;
    }
    static String arg(String[]args,String key,String fallback){for(int i=0;i<args.length-1;i++)if(args[i].equals(key))return args[i+1];if(fallback==null)throw new IllegalArgumentException("Missing "+key);return fallback;}
    static String json(Object value){
        if(value==null)return "null";if(value instanceof String s)return "\""+s.replace("\\","\\\\").replace("\"","\\\"").replace("\n","\\n").replace("\r","\\r").replace("\t","\\t")+"\"";
        if(value instanceof Number||value instanceof Boolean)return value.toString();
        if(value instanceof Map<?,?> m)return "{"+String.join(",",m.entrySet().stream().map(e->json(e.getKey().toString())+":"+json(e.getValue())).toList())+"}";
        if(value instanceof Collection<?> c)return "["+String.join(",",c.stream().map(TraceCapture12111::json).toList())+"]";
        throw new IllegalArgumentException("Cannot serialize "+value);
    }

    /** Resolve only exact mapped signatures; never guess between overloads. */
    static final class Names {
        final Map<String,String> classes=new HashMap<>(), methods=new HashMap<>(), fields=new HashMap<>();
        Names(Path path)throws Exception{
            String owner=null;
            for(String line:Files.readAllLines(path)){
                if(line.isEmpty()||line.startsWith("#"))continue;
                if(!line.startsWith(" ")&&line.endsWith(":")){
                    String[]p=line.substring(0,line.length()-1).split(" -> ");owner=p[0];classes.put(owner,p[1]);continue;
                }
                if(owner==null||!line.contains(" -> "))continue;
                String[]p=line.trim().split(" -> ");String left=p[0].replaceFirst("^\\d+:\\d+:","");
                int space=left.indexOf(' ');if(space<0)continue;String member=left.substring(space+1);
                if(member.contains("(")){int end=member.indexOf(')');methods.put(owner+"#"+member.substring(0,end+1),p[1]);}
                else fields.put(owner+"#"+member,p[1]);
            }
        }
        Class<?> type(String name)throws Exception{return switch(name){case "int"->int.class;case "long"->long.class;case "boolean"->boolean.class;case "void"->void.class;default->Class.forName(classes.getOrDefault(name,name));};}
        Class<?>[] parameters(String signature)throws Exception{if(signature.isEmpty())return new Class<?>[0];String[]p=signature.split(",");Class<?>[]types=new Class<?>[p.length];for(int i=0;i<p.length;i++)types[i]=type(p[i]);return types;}
        Object construct(String owner,String signature,Object...args)throws Exception{Constructor<?> c=type(owner).getDeclaredConstructor(parameters(signature));c.setAccessible(true);return c.newInstance(args);}
        Object field(String owner,String name)throws Exception{String mapped=fields.get(owner+"#"+name);if(mapped==null)throw new NoSuchFieldException(owner+"#"+name);Field f=type(owner).getDeclaredField(mapped);f.setAccessible(true);return f.get(null);}
        Object call(Object receiver,String owner,String name,String signature,Object...args)throws Exception{
            String key=owner+"#"+name+"("+signature+")";String mapped=methods.get(key);if(mapped==null)throw new NoSuchMethodException(key);
            Method m=type(owner).getDeclaredMethod(mapped,parameters(signature));m.setAccessible(true);
            try{return m.invoke(receiver,args);}catch(InvocationTargetException e){throw new RuntimeException(key,e.getCause());}
        }
    }
}
