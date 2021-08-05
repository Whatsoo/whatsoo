drop database whatsoo;
create database whatsoo;
create table whatsoo.user
(
    pk_id bigint auto_increment comment '主键ID'
        primary key,
    uk_username varchar(50) not null comment '用户名',
    uk_email varchar(50) not null comment '邮箱',
    avatar varchar(100) default 'https://avatars3.githubusercontent.com/u/18442141' null comment '头像',
    blog_url varchar(100) null comment '博客网址',
    introduce varchar(500) null comment '自我介绍',
    github_uid varchar(50) null comment 'Github用户名',
    last_login_time datetime not null comment '最后登录时间',
    create_time datetime default current_timestamp() not null on update current_timestamp() comment '创建时间',
    update_time datetime not null comment '更新时间',
    constraint user_blog_url_uindex
        unique (blog_url),
    constraint user_github_uid_uindex
        unique (github_uid),
    constraint user_uk_e_mail_uindex
        unique (uk_email),
    constraint user_uk_username_uindex
        unique (uk_username)
)
    comment '用户表';

create table whatsoo.topic
(
    pk_id bigint auto_increment comment '主键id'
        primary key,
    user_id bigint not null comment '用户id',
    title varchar(100) not null comment '主题标题',
    content varchar(1000) not null comment '帖子内容',
    tags varchar(50) null comment '帖子相关主题id，-分隔，例如1-2-3-4',
    like_times bigint default 0 not null comment '收藏次数',
    click_times bigint default 0 not null comment '点击次数',
    create_time datetime not null comment '创建时间',
    create_user bigint not null comment '创建人',
    update_time datetime not null comment '更新时间',
    update_user bigint not null comment '更新人'
)
    comment '帖子表';

create table whatsoo.tag
(
    pk_id bigint auto_increment comment '主键id'
        primary key,
    tag_name varchar(50) not null comment '主题名称',
    uk_logo varchar(500) default 'https://avatars.githubusercontent.com/u/40875493?s=60&v=4' not null comment 'tag logo',
    associate_tag bigint null comment '关联主题',
    parent_tag bigint not null comment '父主题id，最多两个层级',
    create_time datetime not null comment '创建时间',
    create_user bigint not null comment '创建人',
    constraint topic_topic_name_uindex
        unique (tag_name)
)
    comment '标签表';

create table whatsoo.tag_topic_relation
(
    pk_id bigint auto_increment comment '主键id'
        primary key,
    tag_id bigint not null comment '标签id',
    topic_id int not null comment '主题id',
    create_time int not null comment '创建时间',
    create_user bigint not null comment '创建用户'
)
    comment '主题标签关系表';

create index tag_topic_relation_tag_id_index
	on whatsoo.tag_topic_relation (tag_id);

create or replace table whatsoo.comment
(
	pk_id bigint auto_increment comment '主键id'
		primary key,
	user_id bigint not null comment '用户id',
	post_id bigint not null comment '帖子id',
	content varchar(5000) not null comment '评论内容',
	like_amount int null comment '点赞数量',
	create_time datetime not null comment '创建时间',
	create_user bigint not null comment '创建人'
)
comment '评论表';

create or replace index comment_post_id_index
	on whatsoo.comment (post_id);

create table whatsoo.star
(
    pk_id bigint auto_increment comment '主键id'
        primary key,
    star_type tinyint not null comment '评论点赞-1，帖子收藏-2，用户关注-3',
    user_id bigint not null comment '关注用户id',
    star_id bigint not null comment '被关注id',
    create_time datetime not null comment '创建时间',
    constraint star_star_type_uindex
        unique (star_type),
    constraint star_star_type_user_id_star_id_uindex
        unique (star_type, user_id, star_id)
)
    comment '综合点赞，收藏，关注表';

create or replace table whatsoo.notice
(
	pk_id bigint auto_increment comment '主键id'
		primary key,
	content varchar(5000) not null comment '通知内容',
	notified_user_id bigint not null comment '待通知人',
	viewed tinyint default 0 not null comment '是否被查看',
	create_time datetime not null comment '创建时间',
	create_user bigint not null comment '创建人'
)
comment '通知表';