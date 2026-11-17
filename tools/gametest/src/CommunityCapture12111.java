package org.nucleation.oracle;

import static org.nucleation.oracle.TraceCapture12111.*;

import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;

/** Bounded, raw-placement community fixture capture. Native DataVersion 4671 only.
 * Shares the pinned reflection/runtime helpers with the original four captures,
 * without changing their contract. JSON is read through the bundled Gson at
 * runtime; javac needs only the JDK. No entity/component or intra-tick oracle.
 */
public final class CommunityCapture12111 {
    static int[] origin, size, minimum, maximum;
    static final int MAX_CELLS = 262144, MAX_TICKS = 5000;

    record Fixture(int x, int y, int z, String state, String nbt) {}
    record Action(int tick, String type, int x, int y, int z, String state) {}

    public static void main(String[] args) {
        try { run(args); }
        catch (Throwable error) { error.printStackTrace(); System.exit(1); }
    }

    @SuppressWarnings("unchecked")
    static Map<String,Object> object(Object value, String label) {
        if (!(value instanceof Map<?,?> map)) throw new IllegalArgumentException(label + " must be an object");
        return (Map<String,Object>)map;
    }
    static List<?> array(Object value, String label) {
        if (!(value instanceof List<?> list)) throw new IllegalArgumentException(label + " must be an array");
        return list;
    }
    static String string(Object value, String label) {
        if (!(value instanceof String s) || s.isEmpty()) throw new IllegalArgumentException(label + " must be a nonempty string");
        return s;
    }
    static int integer(Object value, String label) {
        if (!(value instanceof Number n) || !Double.isFinite(n.doubleValue()) || n.doubleValue() != n.intValue())
            throw new IllegalArgumentException(label + " must be an exact int32");
        return n.intValue();
    }
    static int[] vector(Object value, String label) {
        List<?> a=array(value,label);
        if(a.size()!=3)throw new IllegalArgumentException(label+" needs three coordinates");
        return new int[]{integer(a.get(0),label),integer(a.get(1),label),integer(a.get(2),label)};
    }
    static List<Integer> row(int[] p) {return List.of(p[0],p[1],p[2]);}
    static Object integerJson(Object value) {
        // Gson's untyped Map defaults to Double. Preserve schema integers in
        // emitted JSON so downstream integer deserializers need no coercion.
        if(value instanceof Number n && Double.isFinite(n.doubleValue()) && n.doubleValue()==n.intValue())return n.intValue();
        if(value instanceof List<?> list){List<Object> result=new ArrayList<>();for(Object item:list)result.add(integerJson(item));return result;}
        if(value instanceof Map<?,?> map){Map<String,Object> result=new LinkedHashMap<>();for(var entry:map.entrySet())result.put((String)entry.getKey(),integerJson(entry.getValue()));return result;}
        return value;
    }
    static boolean watched(int[] p) {
        for(int i=0;i<3;i++)if(p[i]<minimum[i]||p[i]>maximum[i])return false;
        return true;
    }
    static String hash(Path path, String algorithm) throws Exception {
        return HexFormat.of().formatHex(MessageDigest.getInstance(algorithm).digest(Files.readAllBytes(path)));
    }
    static Object at(int x,int y,int z) throws Exception {
        return names.construct(POS,"int,int,int",Math.addExact(x,origin[0]),Math.addExact(y,origin[1]),Math.addExact(z,origin[2]));
    }
    static Object read(int x,int y,int z) throws Exception {return names.call(world,LEVEL,"getBlockState",POS,at(x,y,z));}
    static void raw(int x,int y,int z,String descriptor) throws Exception {
        // The original helper's y offset is fixed at 100; translate explicitly.
        rawState(Math.addExact(x,origin[0]),Math.subtractExact(Math.addExact(y,origin[1]),ORIGIN_Y),Math.addExact(z,origin[2]),descriptor);
    }
    static void canonical(String descriptor) throws Exception {
        String loaded=descriptor(state(descriptor));
        if(!loaded.equals(descriptor))throw new IllegalArgumentException("Unknown, incomplete or noncanonical state: "+descriptor+" -> "+loaded);
    }

    static void run(String[] args) throws Exception {
        Path mappings=Path.of(arg(args,"--mappings",null));
        Path universe=Path.of(arg(args,"--universe",null));
        Path out=Path.of(arg(args,"--out",null));
        Path input=Path.of(arg(args,"--scenario",null));
        if(Files.exists(universe)||Files.exists(out))throw new IllegalArgumentException("Use new universe and capture paths");
        if(Files.size(input)>16*1024*1024)throw new IllegalArgumentException("Scenario exceeds 16 MiB");
        Class<?> gsonType=Class.forName("com.google.gson.Gson");
        Object gson=gsonType.getConstructor().newInstance();
        Map<String,Object> scenario=object(integerJson(gsonType.getMethod("fromJson",String.class,Class.class)
            .invoke(gson,Files.readString(input),Map.class)),"scenario");
        if(integer(scenario.get("format_version"),"format_version")!=1)throw new IllegalArgumentException("Unknown scenario format");
        if(integer(scenario.get("data_version"),"data_version")!=4671)throw new IllegalArgumentException("Only native 4671 fixtures are supported; no datafixing");
        String label=string(scenario.get("case"),"case");
        if(!(scenario.get("negative_control") instanceof Boolean))throw new IllegalArgumentException("negative_control must be boolean");
        for(String key:List.of("entities","item_entities"))if(scenario.containsKey(key)&&!array(scenario.get(key),key).isEmpty())throw new IllegalArgumentException("Entity loading is not implemented");
        Map<String,Object> source=object(scenario.get("source"),"source");
        if(!string(source.get("sha256"),"source.sha256").matches("[0-9a-f]{64}"))throw new IllegalArgumentException("Invalid source SHA-256");
        boolean exactStates="exact_states_no_datafix".equals(scenario.get("source_mode"));
        if(scenario.containsKey("source_mode")&&!exactStates)throw new IllegalArgumentException("Unknown source_mode");
        if(integer(source.get("data_version"),"source.data_version")!=4671&&!exactStates)throw new IllegalArgumentException("Non-native source requires explicit exact-state compatibility mode");
        size=vector(scenario.get("size"),"size"); origin=vector(scenario.get("origin"),"origin");
        Map<String,Object> watch=object(scenario.get("watch"),"watch");
        minimum=vector(watch.get("min"),"watch.min"); maximum=vector(watch.get("max"),"watch.max");
        long volume=1;
        for(int i=0;i<3;i++){
            if(size[i]<=0||minimum[i]>0||maximum[i]<size[i]-1)throw new IllegalArgumentException("Watch must contain positive fixture size");
            long extent=(long)maximum[i]-minimum[i]+1;
            if(extent<=0||extent>MAX_CELLS)throw new IllegalArgumentException("Invalid watch extent");
            volume=Math.multiplyExact(volume,extent);
            long lo=(long)origin[i]+minimum[i],hi=(long)origin[i]+maximum[i];
            if(lo < -29999968L || hi > 29999967L)throw new IllegalArgumentException("Watch outside bounded world coordinates");
        }
        if((long)origin[1]+minimum[1]<-64||(long)origin[1]+maximum[1]>=320)throw new IllegalArgumentException("Watch outside overworld build height");
        int ticks=integer(scenario.get("capture_ticks"),"capture_ticks");
        int stride=scenario.containsKey("capture_stride")?integer(scenario.get("capture_stride"),"capture_stride"):1;
        if(ticks<1||ticks>MAX_TICKS)throw new IllegalArgumentException("Capture exceeds bounded simulated-tick budget");
        if(stride<1||stride>ticks)throw new IllegalArgumentException("capture_stride must be positive and at most capture_ticks");
        List<Integer> sampledTicks=new ArrayList<>();
        for(int t=0;t<=ticks;t+=stride)sampledTicks.add(t);
        if(sampledTicks.get(sampledTicks.size()-1)!=ticks)sampledTicks.add(ticks);
        // Retained/scanned cells are bounded independently of simulated ticks.
        // As before, the cell budget counts snapshots after the initial frame.
        if(volume>MAX_CELLS||volume*(sampledTicks.size()-1)>50000000L)throw new IllegalArgumentException("Capture exceeds bounded cell/snapshot budget");
        int seed=integer(scenario.get("rng_seed"),"rng_seed");
        List<Fixture> fixture=new ArrayList<>(); Set<List<Integer>> occupied=new HashSet<>();
        for(Object entry:array(scenario.get("fixture"),"fixture")){
            Map<String,Object> b=object(entry,"fixture block");int[] p=vector(b.get("pos"),"fixture.pos");
            for(int i=0;i<3;i++)if(p[i]<0||p[i]>=size[i])throw new IllegalArgumentException("Fixture outside size");
            if(!occupied.add(row(p)))throw new IllegalArgumentException("Duplicate fixture position: "+row(p));
            String descriptor=string(b.get("state"),"fixture.state");
            String nbt=b.containsKey("nbt")?string(b.get("nbt"),"fixture.nbt"):null;
            if(descriptor.equals("minecraft:air")&&nbt!=null)throw new IllegalArgumentException("Air cannot carry block entity NBT");
            if(exactStates&&nbt!=null)throw new IllegalArgumentException("Exact-state compatibility mode cannot import block entities");
            fixture.add(new Fixture(p[0],p[1],p[2],descriptor,nbt));
        }
        if(fixture.isEmpty())throw new IllegalArgumentException("Empty fixture");
        List<Action> actions=new ArrayList<>();int priorTick=0;
        for(Object entry:array(scenario.get("actions"),"actions")){
            Map<String,Object> a=object(entry,"action");int t=integer(a.get("tick"),"action.tick");
            int[] p=vector(a.get("pos"),"action.pos");String kind=string(a.get("type"),"action.type");
            if(t<1||t>ticks||t<priorTick||!watched(p))throw new IllegalArgumentException("Actions must be ordered, within ticks and watch box");
            priorTick=t;String value=null;
            if(kind.equals("set_block"))value=string(a.get("state"),"action.state");
            else if(!Set.of("notify_neighbors","pull_lever","use_note_block").contains(kind)||a.containsKey("state"))throw new IllegalArgumentException("Unknown or invalid action: "+kind);
            actions.add(new Action(t,kind,p[0],p[1],p[2],value));
        }

        String mappingHash=hash(mappings,"SHA-1");
        if(!mappingHash.equals("5621e9253f05fd57872bbe7f8ddf5f9a7d525955"))throw new IllegalArgumentException("Unexpected 1.21.11 mappings");
        names=new Names(mappings);
        Path loaded=Path.of(names.type("net.minecraft.SharedConstants").getProtectionDomain().getCodeSource().getLocation().toURI());
        String loadedHash=hash(loaded,"SHA-256");
        if(!loadedHash.equals("ec47239a8de246335e1d54f6ac319bd35641778eb4b6a6da06372840d02fcebc"))throw new IllegalArgumentException("Unexpected loaded server");
        names.call(null,"net.minecraft.SharedConstants","tryDetectVersion","");
        names.call(null,"net.minecraft.server.Bootstrap","bootStrap","");
        names.call(null,"net.minecraft.util.Util","startTimerHackThread","");
        Object storage=names.call(null,"net.minecraft.world.level.storage.LevelStorageSource","createDefault","java.nio.file.Path",universe);
        Object access=names.call(storage,"net.minecraft.world.level.storage.LevelStorageSource","createAccess","java.lang.String","gametestworld");
        Object packs=names.call(null,"net.minecraft.server.packs.repository.ServerPacksSource","createPackRepository","net.minecraft.world.level.storage.LevelStorageSource$LevelStorageAccess",access);
        server=names.call(null,"net.minecraft.gametest.framework.GameTestServer","create","java.lang.Thread,net.minecraft.world.level.storage.LevelStorageSource$LevelStorageAccess,net.minecraft.server.packs.repository.PackRepository,java.util.Optional,boolean",Thread.currentThread(),access,packs,Optional.empty(),false);
        boolean success=false;
        try{
            names.call(server,MC,"initServer","");world=names.call(server,MC,"overworld","");
            registry=names.call(world,LEVEL,"registryAccess","");
            blocks=names.call(registry,PROVIDER,"lookupOrThrow","net.minecraft.resources.ResourceKey",names.field("net.minecraft.core.registries.Registries","BLOCK"));
            for(Fixture b:fixture)canonical(b.state);
            for(Action a:actions)if(a.state!=null)canonical(a.state);
            int warmup=warmWatch();
            for(int y=minimum[1];y<=maximum[1];y++)for(int z=minimum[2];z<=maximum[2];z++)for(int x=minimum[0];x<=maximum[0];x++){
                names.call(world,LEVEL,"removeBlockEntity",POS,at(x,y,z));raw(x,y,z,"minecraft:air");
            }
            for(Fixture b:fixture)raw(b.x,b.y,b.z,b.state);
            for(Fixture b:fixture)if(b.nbt!=null){
                Object entity=names.call(null,ENTITY,"loadStatic",POS+","+STATE+","+TAG+","+PROVIDER,at(b.x,b.y,b.z),state(b.state),compound(b.nbt),registry);
                if(entity==null)throw new IllegalStateException("Block entity failed to load: "+b);
                names.call(world,LEVEL,"setBlockEntity",ENTITY,entity);
            }
            // Detached: never connected, ticked or added to the level. NoteBlock's
            // actual use path needs a player for its statistic/game-event calls.
            Object notePlayer=null;
            if(actions.stream().anyMatch(a->a.type.equals("use_note_block"))){
                Object profile=names.construct("com.mojang.authlib.GameProfile","java.util.UUID,java.lang.String",UUID.fromString("00000000-0000-0000-0000-000000000001"),"oracle-note-use");
                Object info=names.call(null,"net.minecraft.server.level.ClientInformation","createDefault","");
                notePlayer=names.construct("net.minecraft.server.level.ServerPlayer",MC+","+SERVER_LEVEL+",com.mojang.authlib.GameProfile,net.minecraft.server.level.ClientInformation",server,world,profile,info);
            }
            Object random=names.call(world,LEVEL,"getRandom","");
            names.call(random,"net.minecraft.util.RandomSource","setSeed","long",(long)seed);
            Map<String,Object> initial=capture(0);
            long expected=fixture.stream().filter(b->!b.state.equals("minecraft:air")).count();
            require(((List<?>)initial.get("blocks")).size()==expected,"Initial non-air count mismatch");
            for(Fixture b:fixture)require(descriptor(read(b.x,b.y,b.z)).equals(b.state),"Initial state mismatch: "+b);
            List<Map<String,Object>> snapshots=new ArrayList<>();int next=0;
            for(int t=1;t<=ticks;t++){
                while(next<actions.size()&&actions.get(next).tick==t){
                    Action a=actions.get(next++);Object p=at(a.x,a.y,a.z);
                    if(a.type.equals("set_block"))names.call(world,LEVEL,"setBlock",POS+","+STATE+",int",p,state(a.state),3);
                    else if(a.type.equals("pull_lever")){
                        Object current=read(a.x,a.y,a.z);
                        require(descriptor(current).startsWith("minecraft:lever["),"pull_lever target is not a lever: "+a);
                        names.call(names.field("net.minecraft.world.level.block.Blocks","LEVER"),"net.minecraft.world.level.block.LeverBlock","pull",STATE+","+LEVEL+","+POS+",net.minecraft.world.entity.player.Player",current,world,p,null);
                    }else if(a.type.equals("use_note_block")){
                        Object current=read(a.x,a.y,a.z);
                        require(descriptor(current).startsWith("minecraft:note_block["),"use_note_block target is not a note block: "+a);
                        names.call(names.field("net.minecraft.world.level.block.Blocks","NOTE_BLOCK"),"net.minecraft.world.level.block.NoteBlock","useWithoutItem",STATE+","+LEVEL+","+POS+",net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult",current,world,p,notePlayer,null);
                    }else names.call(world,LEVEL,"updateNeighborsAt",POS+",net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation",p,names.field("net.minecraft.world.level.block.Blocks","AIR"),null);
                }
                tick();
                if(t%stride==0||t==ticks)snapshots.add(capture(t));
            }
            Map<String,Object> result=new LinkedHashMap<>(scenario);
            result.put("mc_version","1.21.11");result.put("settle","in_world");
            result.put("action_boundary","immediately before the numbered tick; initial snapshot is tick zero");
            result.put("scenario_sha256",hash(input,"SHA-256"));result.put("warmup_ticks",warmup);
            result.put("mappings_sha1",mappingHash);result.put("server_inner_sha256",loadedHash);
            result.put("java_runtime_version",System.getProperty("java.runtime.version"));
            result.put("server_kind","Minecraft GameTestServer; no client or connected players");
            result.put("scope","tick-end blocks, comparator outputs and simple occupied slot IDs/counts; no entity, item-component, moving-BE or intra-tick equality");
            if(scenario.containsKey("capture_stride"))result.put("sampled_ticks",sampledTicks);
            if(stride>1)result.put("scope","sampled tick-end blocks, comparator outputs and simple occupied slot IDs/counts; every simulated tick and action executed, but intermediate states are not captured; no entity, item-component, moving-BE or intra-tick equality");
            if(stride>1)result.put("final_activity",finalActivity(ticks,snapshots.get(snapshots.size()-1)));
            result.put("initial",initial);result.put("snapshots",snapshots);
            if(out.toAbsolutePath().getParent()!=null)Files.createDirectories(out.toAbsolutePath().getParent());
            // Gson also escapes all JSON control characters in arbitrary source metadata/NBT.
            // Large mechanisms already retain their snapshots. Stream JSON to
            // avoid allocating a second, hundreds-of-megabytes output string.
            try(var writer=Files.newBufferedWriter(out,StandardOpenOption.CREATE_NEW)){
                gsonType.getMethod("toJson",Object.class,Appendable.class).invoke(gson,result,writer);
                writer.write("\n");
            }
            System.out.println("CAPTURED "+out+" ticks="+ticks+" players=0 case="+label);success=true;
        }finally{names.call(server,MC,"stopServer","");}
        if(success)System.exit(0);
    }

    static Map<String,Object> finalActivity(int tick,Map<String,Object> snapshot) throws Exception {
        Map<String,Object> result=new LinkedHashMap<>();result.put("tick",tick);
        for(String kind:List.of("block","fluid")){
            Object ticks=names.call(world,SERVER_LEVEL,kind.equals("block")?"getBlockTicks":"getFluidTicks","");
            result.put(kind+"_ticks",names.call(ticks,"net.minecraft.world.ticks.LevelTicks","count",""));
        }
        for(String field:List.of("blockEvents","blockEventsToReschedule")){
            String mapped=names.fields.get(SERVER_LEVEL+"#"+field);
            if(mapped==null)throw new NoSuchFieldException(SERVER_LEVEL+"#"+field);
            var member=names.type(SERVER_LEVEL).getDeclaredField(mapped);member.setAccessible(true);
            result.put(field.equals("blockEvents")?"block_events":"rescheduled_block_events",((Collection<?>)member.get(world)).size());
        }
        int moving=0;
        for(Object entry:array(snapshot.get("blocks"),"final blocks")){
            List<?> row=array(entry,"block row");
            if(string(row.get(3),"block state").startsWith("minecraft:moving_piston["))moving++;
        }
        result.put("moving_pistons_in_watch",moving);
        result.put("scope","Queue counts cover the server level; moving-piston count covers only the declared watch. No runtime-entity census.");
        return result;
    }

    static int warmWatch() throws Exception {
        Object cache=names.call(world,SERVER_LEVEL,"getChunkSource","");List<Object> required=new ArrayList<>();
        int minX=(origin[0]+minimum[0])>>4,maxX=(origin[0]+maximum[0])>>4;
        int minZ=(origin[2]+minimum[2])>>4,maxZ=(origin[2]+maximum[2])>>4;
        if((long)(maxX-minX+3)*(maxZ-minZ+3)>1024)throw new IllegalArgumentException("Too many chunk tickets");
        for(int x=minX-1;x<=maxX+1;x++)for(int z=minZ-1;z<=maxZ+1;z++){
            Object chunk=names.construct("net.minecraft.world.level.ChunkPos","int,int",x,z);
            if(x>=minX&&x<=maxX&&z>=minZ&&z<=maxZ)required.add(chunk);
            for(String ticket:List.of("PLAYER_SIMULATION","FORCED"))names.call(cache,"net.minecraft.server.level.ServerChunkCache","addTicketWithRadius","net.minecraft.server.level.TicketType,net.minecraft.world.level.ChunkPos,int",names.field("net.minecraft.server.level.TicketType",ticket),chunk,2);
            names.call(world,SERVER_LEVEL,"setChunkForced","int,int,boolean",x,z,true);
        }
        for(int n=0;n<600;n++){
            boolean ready=true;
            for(Object chunk:required){long packed=(long)names.call(chunk,"net.minecraft.world.level.ChunkPos","toLong","");ready&=(boolean)names.call(world,SERVER_LEVEL,"isPositionTickingWithEntitiesLoaded","long",packed);}
            if(ready)return n;tick();
        }
        throw new IllegalStateException("Observation chunks never became entity-ticking ready");
    }

    static Map<String,Object> capture(int tick) throws Exception {
        int players=(int)names.call(server,MC,"getPlayerCount","");require(players==0,"Capture requires zero players");
        List<Object> blockRows=new ArrayList<>(),inventories=new ArrayList<>(),outputs=new ArrayList<>();
        Object items=names.field("net.minecraft.core.registries.BuiltInRegistries","ITEM");
        Class<?> container=names.type("net.minecraft.world.Container"),comparator=names.type("net.minecraft.world.level.block.entity.ComparatorBlockEntity");
        for(int y=minimum[1];y<=maximum[1];y++)for(int z=minimum[2];z<=maximum[2];z++)for(int x=minimum[0];x<=maximum[0];x++){
            Object p=at(x,y,z);String value=descriptor(names.call(world,LEVEL,"getBlockState",POS,p));if(value.equals("minecraft:air"))continue;
            blockRows.add(List.of(x,y,z,value));Object be=names.call(world,LEVEL,"getBlockEntity",POS,p);
            if(be!=null&&comparator.isInstance(be))outputs.add(List.of(x,y,z,(int)names.call(be,"net.minecraft.world.level.block.entity.ComparatorBlockEntity","getOutputSignal","")));
            if(be!=null&&container.isInstance(be)){
                int slots=(int)names.call(be,"net.minecraft.world.Container","getContainerSize","");
                for(int slot=0;slot<slots;slot++){
                    Object stack=names.call(be,"net.minecraft.world.Container","getItem","int",slot);
                    if((boolean)names.call(stack,"net.minecraft.world.item.ItemStack","isEmpty",""))continue;
                    Object item=names.call(stack,"net.minecraft.world.item.ItemStack","getItem","");
                    String id=names.call(items,"net.minecraft.core.Registry","getKey","java.lang.Object",item).toString();
                    inventories.add(List.of(x,y,z,slot,id,(int)names.call(stack,"net.minecraft.world.item.ItemStack","getCount","")));
                }
            }
        }
        Map<String,Object> row=new LinkedHashMap<>();row.put("tick",tick);row.put("players",players);row.put("blocks",blockRows);row.put("inventories",inventories);row.put("comparator_outputs",outputs);return row;
    }
}
