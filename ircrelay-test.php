
<?php

require_once "./inc/functions.php";

    $thread_target = 9690;
    $board_target = "test";
    $name = "IRCSageru #test";
    openBoard($board_target);

    if ($_SERVER['REQUEST_METHOD'] === 'GET') {
        echo "https://kissu.moe/$board_target/thread/$thread_target";
        return;
    }

    if (!isset($_POST["pass"])){
        die("Pass is missing");
    }
    if ($_POST["pass"] != "12345"){
        die("Pass is incorrect");
    }
    
    if (!isset($_POST["posts"]) || strlen($_POST["posts"]) == 0){
        die("Posts is missing");
    }
    $posts_list = explode("\0" , $_POST["posts"]);

    foreach($posts_list as $key=>$post ){
        $posts_list[$key] = "" . substr($post , strpos($post , ":", 1) + 1);
    }

    
    foreach($posts_list as $post ){		
        var_dump($post);
		$post_clean = "" . preg_replace("/cuck|nigger|incel/i", "-", $post);
        $query = prepare(sprintf("INSERT INTO ``posts_%s``(id, thread, subject, email, name, trip , capcode, body,
            body_nomarkup, time, bump, files, num_files, filehash, password, ip, sticky, locked, cycle, sage, embed, slug, wl_token, zombie) VALUES (
            NULL, :thread, :subject, :email, :name, :trip, :capcode, :body, :body_nomarkup,
            :time, :time, :files, :num_files, :filehash, :password, :ip, :sticky, :locked,
            :cycle, 0, :embed, :slug, :wl_token, 0)", $board_target ));

        $query->bindValue(':subject', null, PDO::PARAM_NULL);
        $query->bindValue(':email', null, PDO::PARAM_NULL);
        $query->bindValue(':trip', null, PDO::PARAM_NULL);

        $query->bindValue(':name', $name);
        $query->bindValue(':body_nomarkup', "$post_clean");
        markup($post_clean);
        $query->bindValue(':body', $post_clean);
        $query->bindValue(':time', time(), PDO::PARAM_INT);
        $query->bindValue(':password', "c9d030dldf" );
        $query->bindValue(':ip', $_SERVER['REMOTE_ADDR']);
        $query->bindValue(':capcode', "Bot", PDO::PARAM_STR);
        $query->bindValue(':thread', $thread_target, PDO::PARAM_INT);
        
        $query->bindValue(':sticky', false, PDO::PARAM_INT);
        $query->bindValue(':locked', false, PDO::PARAM_INT);
        $query->bindValue(':cycle', false, PDO::PARAM_INT);
        $query->bindValue(':embed', null, PDO::PARAM_NULL);
        $query->bindValue(':wl_token', null, PDO::PARAM_NULL);
        $query->bindValue(':files', null, PDO::PARAM_NULL);
        $query->bindValue(':num_files', 0);
        $query->bindValue(':filehash', null, PDO::PARAM_NULL);
        $query->bindValue(':slug', NULL);

        
        if (!$query->execute()) {
            echo "POST ERR: " . error(db_error($query));
        } else{
            $id = $pdo->lastInsertId();
            Hazuki::rebuildProperties($id, $thread_target,  $board_target, "");
            Hazuki::rebuildSummary($id , $board_target , "");
        }
    }
    
    bumpThread($thread_target);
    
    Hazuki::rebuildThread($thread_target, $board_target);
    Hazuki::rebuildCatalog($thread_target , $board_target , "");
    
    Hazuki::rebuildHome($thread_target , $board_target , "");
    Hazuki::rebuildOverboard($thread_target , $board_target , "");

    Hazuki::send(false);

    buildThread($thread_target, false, false, true);

    rebuildThemes('post', $board_target);

    buildIndex();
    // insert into board & thread